// abogen-web 没有真正的 CLI/单次调用接口，只能把它当作本地 HTTP 服务来驱动：
// 1) POST /wizard/text 创建 pending job
// 2) POST /wizard/update (step=book) 设置语言/音色/语速
// 3) POST /wizard/update (step=chapters) 勾选章节，推进到 entities
// 4) POST /wizard/finish 提交 job（必须再次带上 chapter-0-enabled=on，
//    否则 wizard_finish 内部会重新跑一遍 apply_prepare_form 并把 enabled 覆盖回 false，
//    导致任务以 "No chapters were enabled" 报错——这个坑已经用本机 abogen-web 实测过）
// 5) 轮询 GET /jobs/{job_id} 页面里的 `badge badge--{status}` class 判断状态
//    （abogen 文档提到的 GET /api/jobs/<id> JSON 接口在当前版本源码里并不存在）
// 6) GET /jobs/{job_id}/download/audio 拿到最终 wav 字节，交给 rodio 播放

use rodio::Decoder;
use std::io::Cursor;
use tokio::task;
use tokio::time::{sleep, Duration};

pub struct AbogenSpeakOptions {
    pub language: String,
    pub voice: String,
    pub speed: f32,
    pub base_url: String,
}

impl Default for AbogenSpeakOptions {
    fn default() -> Self {
        AbogenSpeakOptions {
            language: "z".into(),
            voice: "zf_xiaoyi".into(),
            speed: 1.2,
            base_url: "http://127.0.0.1:8808".into(),
        }
    }
}

pub async fn synthesize_and_play(text: &str, options: AbogenSpeakOptions) -> Result<(), String> {
    let client = reqwest::Client::new();
    let base = options.base_url.trim_end_matches('/').to_string();

    let pending_id = submit_text(&client, &base, text).await?;
    configure_book_step(&client, &base, &pending_id, &options).await?;
    advance_chapters_step(&client, &base, &pending_id).await?;
    let job_id = finish_job(&client, &base, &pending_id).await?;

    wait_for_completion(&client, &base, &job_id).await?;

    let audio = download_audio(&client, &base, &job_id).await?;
    println!("abogen audio bytes: {}", audio.len());
    play_audio_from_vec(audio).await;

    Ok(())
}

async fn submit_text(client: &reqwest::Client, base: &str, text: &str) -> Result<String, String> {
    let form = reqwest::multipart::Form::new()
        .text("text", text.to_string())
        .text("title", "hello_cargo_tts".to_string());

    let res = client
        .post(format!("{base}/wizard/text"))
        .header("X-Abogen-Wizard", "json")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("提交文本失败: {e}"))?;

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("解析 wizard/text 响应失败: {e}"))?;

    json.get("pending_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("wizard/text 响应缺少 pending_id: {json}"))
}

async fn configure_book_step(
    client: &reqwest::Client,
    base: &str,
    pending_id: &str,
    options: &AbogenSpeakOptions,
) -> Result<(), String> {
    let form = reqwest::multipart::Form::new()
        .text("pending_id", pending_id.to_string())
        .text("step", "book")
        .text("next_step", "chapters")
        .text("language", options.language.clone())
        .text("voice", options.voice.clone())
        .text("speed", options.speed.to_string());

    let res = client
        .post(format!("{base}/wizard/update"))
        .header("X-Abogen-Wizard", "json")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("配置语音参数失败: {e}"))?;

    check_wizard_error(res).await
}

async fn advance_chapters_step(
    client: &reqwest::Client,
    base: &str,
    pending_id: &str,
) -> Result<(), String> {
    let form = reqwest::multipart::Form::new()
        .text("pending_id", pending_id.to_string())
        .text("step", "chapters")
        .text("next_step", "entities")
        .text("chunk_level", "paragraph")
        .text("chapter-0-enabled", "on");

    let res = client
        .post(format!("{base}/wizard/update"))
        .header("X-Abogen-Wizard", "json")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("推进章节步骤失败: {e}"))?;

    check_wizard_error(res).await
}

async fn finish_job(
    client: &reqwest::Client,
    base: &str,
    pending_id: &str,
) -> Result<String, String> {
    // chapter-0-enabled 必须再传一次：wizard/finish 内部会重新执行
    // apply_prepare_form，若这里缺失该字段，章节会被重新标记为 disabled。
    let form = reqwest::multipart::Form::new()
        .text("pending_id", pending_id.to_string())
        .text("chapter-0-enabled", "on");

    let res = client
        .post(format!("{base}/wizard/finish"))
        .header("X-Abogen-Wizard", "json")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("提交任务失败: {e}"))?;

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("解析 wizard/finish 响应失败: {e}"))?;

    if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
        if !err.is_empty() {
            return Err(format!("abogen 任务提交出错: {err}"));
        }
    }

    json.get("job_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("wizard/finish 响应缺少 job_id: {json}"))
}

async fn check_wizard_error(res: reqwest::Response) -> Result<(), String> {
    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("解析 wizard/update 响应失败: {e}"))?;

    match json.get("error").and_then(|v| v.as_str()) {
        Some(err) if !err.is_empty() => Err(format!("abogen 配置步骤出错: {err}")),
        _ => Ok(()),
    }
}

async fn wait_for_completion(
    client: &reqwest::Client,
    base: &str,
    job_id: &str,
) -> Result<(), String> {
    const MAX_ATTEMPTS: u32 = 150; // 150 * 2s = 5 分钟超时
    for _ in 0..MAX_ATTEMPTS {
        let html = client
            .get(format!("{base}/jobs/{job_id}"))
            .send()
            .await
            .map_err(|e| format!("查询任务状态失败: {e}"))?
            .text()
            .await
            .map_err(|e| format!("读取任务状态页面失败: {e}"))?;

        if html.contains("badge badge--completed") {
            return Ok(());
        }
        if html.contains("badge badge--failed") {
            return Err(format!("abogen 任务 {job_id} 转换失败"));
        }
        if html.contains("badge badge--cancelled") {
            return Err(format!("abogen 任务 {job_id} 已被取消"));
        }

        sleep(Duration::from_secs(2)).await;
    }

    Err(format!("等待 abogen 任务 {job_id} 完成超时"))
}

async fn download_audio(
    client: &reqwest::Client,
    base: &str,
    job_id: &str,
) -> Result<Vec<u8>, String> {
    let res = client
        .get(format!("{base}/jobs/{job_id}/download/audio"))
        .send()
        .await
        .map_err(|e| format!("下载音频失败: {e}"))?;

    if !res.status().is_success() {
        return Err(format!("下载音频失败，HTTP 状态: {}", res.status()));
    }

    res.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("读取音频字节失败: {e}"))
}

async fn play_audio_from_vec(audio_data: Vec<u8>) {
    let res = task::spawn_blocking(move || {
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        let cursor = Cursor::new(audio_data);
        let source = Decoder::new(cursor).unwrap();

        sink.append(source);
        sink.sleep_until_end();
    });
    res.await.unwrap();
}
