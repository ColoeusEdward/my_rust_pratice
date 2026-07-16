// x86_ubuntu 分支：仅保留“获取与播放音频”相关逻辑。
// 保留接口：
//   - play_text      : POST /playText     文本经 Edge TTS 合成后在本机播放
//   - get_text_audio : POST /getTextAudio 文本合成为音频字节后直接返回给网页
// 其余功能（charge/brave/potplay/toast/check_network/play_bingbong/Alt+B 热键等）
// 依赖 winapi / enigo / rsautogui / screenshots / PotPlayer / PowerShell，均为 Windows
// 专有，已在本分支移除，以便在 x86 Ubuntu 上编译运行。

use edge_tts_rust::Boundary;
use edge_tts_rust::EdgeTtsClient;
use edge_tts_rust::SpeakOptions;
use serde::Deserialize;
use tokio::task;
use warp::Rejection;

use rodio::Decoder;
use std::io::Cursor;

#[derive(Deserialize)]
pub struct PlayTextData {
    str: String,
}

pub async fn play_text(dat: PlayTextData) -> Result<String, Rejection> {
    // dat: PlayTextData
    let res = tokio::spawn(async move {
        let client = EdgeTtsClient::new().unwrap();
        let result = client
            .synthesize(
                dat.str.as_str(),
                // "我早已麻痹",
                SpeakOptions {
                    voice: "zh-CN-XiaoxiaoNeural".into(),
                    boundary: Boundary::Sentence,
                    rate: "+20%".into(),
                    volume: "-10%".into(),
                    ..SpeakOptions::default()
                },
            )
            .await
            .unwrap();

        println!("audio bytes: {}", result.audio.len());
        println!("boundaries: {}", result.boundaries.len());
        play_audio_from_vec(result.audio).await;
        // Ok(())
    });

    match res.await {
        Ok(res) => Ok(format!("play_text 成功")),
        Err(e) => {
            println!("play_text 失败: {}", e);
            return Ok(format!("play_text 失败"));
        }
    }
}

// 仿照 play_text：合成语音后不播放，直接把音频字节返回给发起请求的网页
pub async fn get_text_audio(dat: PlayTextData) -> Result<impl warp::Reply, Rejection> {
    println!(
        "get_text_audio 收到请求: 字符数={}, 内容={:?}",
        dat.str.chars().count(),
        dat.str
    );
    // 空文本/纯空白会让 Edge TTS 返回 NoAudioReceived（500），提前拦截
    if dat.str.trim().is_empty() {
        let reply = warp::http::Response::builder()
            .status(warp::http::StatusCode::BAD_REQUEST)
            .header("content-type", "text/plain; charset=utf-8")
            .body("get_text_audio 失败: str 为空".to_string().into_bytes())
            .unwrap();
        return Ok(reply);
    }
    let client = EdgeTtsClient::new().unwrap();
    let result = client
        .synthesize(
            dat.str.as_str(),
            SpeakOptions {
                voice: "zh-CN-XiaoxiaoNeural".into(),
                boundary: Boundary::Sentence,
                rate: "+20%".into(),
                volume: "-10%".into(),
                ..SpeakOptions::default()
            },
        )
        .await;

    match result {
        Ok(res) => {
            println!("get_text_audio 音频字节: {}", res.audio.len());
            let reply = warp::http::Response::builder()
                .header("content-type", "audio/mpeg")
                .header("content-length", res.audio.len())
                .body(res.audio)
                .unwrap();
            Ok(reply)
        }
        Err(e) => {
            println!("get_text_audio 失败: {:?}", e);
            let body = format!("get_text_audio 失败: {:?}", e).into_bytes();
            let reply = warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .header("content-type", "text/plain; charset=utf-8")
                .body(body)
                .unwrap();
            Ok(reply)
        }
    }
}

async fn play_audio_from_vec(audio_data: Vec<u8>) {
    let res = task::spawn_blocking(move || {
        // 1. 获取默认输出设备的句柄
        // _stream 必须保持存活，否则声音会立即停止
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        // 3. 将 Vec<u8> 包装在 Cursor 中，因为它需要实现 Read + Seek
        let cursor = Cursor::new(audio_data);

        // 4. 解码音频数据（自动识别 MP3, WAV, Vorbis, Flac 等）
        let source = Decoder::new(cursor).unwrap();

        // 5. 将音频源放入 Sink 播放
        sink.append(source);

        // 6. 阻塞当前线程直到音频播放完毕（否则函数结束释放资源声音就没了）
        // 注：原 Windows 分支在此处启动 Alt+B 热键监听（winapi GetAsyncKeyState）来
        //     暂停/恢复播放，Linux 上不可用，已移除。
        sink.sleep_until_end();
    });
    res.await.unwrap();
}
