use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

const UMI_OCR_PADDLE_EXE: &str = r"D:\Software\umi_orc\Umi-OCR_Paddle_v2.1.5\UmiOCR-data\plugins\win7_x64_PaddleOCR-json\PaddleOCR-json.exe";

trait OcrProcess {
    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>>;
    fn kill(&mut self) -> std::io::Result<()>;
    fn wait(&mut self) -> std::io::Result<ExitStatus>;
}

impl OcrProcess for Child {
    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        Child::try_wait(self)
    }

    fn kill(&mut self) -> std::io::Result<()> {
        Child::kill(self)
    }

    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        Child::wait(self)
    }
}

struct OcrProcessGuard<P: OcrProcess> {
    child: P,
}

impl<P: OcrProcess> OcrProcessGuard<P> {
    fn new(child: P) -> Self {
        Self { child }
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }
}

impl<P: OcrProcess> Drop for OcrProcessGuard<P> {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn run_umi_ocr(image_path: &Path) -> Result<String, String> {
    let mut engine = PaddleOcrEngine::start()?;
    engine.recognize(image_path)
}

struct PaddleOcrEngine {
    process: OcrProcessGuard<Child>,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl PaddleOcrEngine {
    fn start() -> Result<Self, String> {
        let exe_path = PathBuf::from(UMI_OCR_PADDLE_EXE);
        if !exe_path.exists() {
            return Err(format!(
                "未找到 Umi-OCR Paddle 引擎: {}",
                exe_path.display()
            ));
        }

        let cwd = exe_path
            .parent()
            .ok_or_else(|| format!("无法获取 Umi-OCR Paddle 引擎目录: {}", exe_path.display()))?;
        let mut child = Command::new(&exe_path)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("启动 Umi-OCR Paddle 引擎失败: {}", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "无法打开 Umi-OCR Paddle 引擎 stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "无法打开 Umi-OCR Paddle 引擎 stdout".to_string())?;
        let mut engine = Self {
            process: OcrProcessGuard::new(child),
            stdin,
            stdout: BufReader::new(stdout),
        };

        engine.wait_until_ready()?;
        Ok(engine)
    }

    fn wait_until_ready(&mut self) -> Result<(), String> {
        loop {
            if let Some(status) = self.process.try_wait().map_err(|e| e.to_string())? {
                return Err(format!("Umi-OCR Paddle 引擎初始化失败，退出码: {}", status));
            }

            let mut line = String::new();
            let bytes = self
                .stdout
                .read_line(&mut line)
                .map_err(|e| format!("读取 Umi-OCR Paddle 引擎初始化输出失败: {}", e))?;
            if bytes == 0 {
                return Err("Umi-OCR Paddle 引擎初始化输出已关闭".to_string());
            }
            if line.contains("OCR init completed.") {
                return Ok(());
            }
        }
    }

    fn recognize(&mut self, image_path: &Path) -> Result<String, String> {
        let request = serde_json::json!({ "image_path": image_path.to_string_lossy() });
        writeln!(self.stdin, "{}", request)
            .and_then(|_| self.stdin.flush())
            .map_err(|e| format!("向 Umi-OCR Paddle 引擎发送识别请求失败: {}", e))?;

        let mut response = String::new();
        let bytes = self
            .stdout
            .read_line(&mut response)
            .map_err(|e| format!("读取 Umi-OCR Paddle 引擎识别结果失败: {}", e))?;
        if bytes == 0 {
            return Err("Umi-OCR Paddle 引擎识别结果输出已关闭".to_string());
        }

        paddle_ocr_response_to_text(&response)
    }
}

fn paddle_ocr_response_to_text(response: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(response.trim())
        .map_err(|e| format!("解析 Umi-OCR Paddle 识别结果失败: {}", e))?;
    let code = value
        .get("code")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| "Umi-OCR Paddle 识别结果缺少 code".to_string())?;

    match code {
        100 => paddle_ocr_data_to_text(
            value
                .get("data")
                .ok_or_else(|| "Umi-OCR Paddle 识别结果缺少 data".to_string())?,
        ),
        101 => Ok(String::new()),
        _ => {
            let data = value
                .get("data")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("未知错误");
            Err(format!("PaddleOCR-json 识别失败(code={}): {}", code, data))
        }
    }
}

fn paddle_ocr_data_to_text(data: &serde_json::Value) -> Result<String, String> {
    if let Some(text) = data.as_str() {
        return Ok(text.to_string());
    }

    let lines = data
        .as_array()
        .ok_or_else(|| "Umi-OCR Paddle 识别结果 data 格式不正确".to_string())?
        .iter()
        .filter_map(|item| item.get("text").and_then(serde_json::Value::as_str))
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::os::windows::process::ExitStatusExt;
    use std::process::ExitStatus;
    use std::rc::Rc;

    #[derive(Default)]
    struct FakeOcrProcess {
        calls: Rc<RefCell<Vec<&'static str>>>,
    }

    impl OcrProcess for FakeOcrProcess {
        fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
            self.calls.borrow_mut().push("try_wait");
            Ok(None)
        }

        fn kill(&mut self) -> std::io::Result<()> {
            self.calls.borrow_mut().push("kill");
            Ok(())
        }

        fn wait(&mut self) -> std::io::Result<ExitStatus> {
            self.calls.borrow_mut().push("wait");
            Ok(ExitStatus::from_raw(0))
        }
    }

    #[test]
    fn dropping_ocr_process_guard_kills_and_waits_for_child_process() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let fake_process = FakeOcrProcess {
            calls: Rc::clone(&calls),
        };

        {
            let _guard = OcrProcessGuard::new(fake_process);
        }

        assert_eq!(&*calls.borrow(), &["kill", "wait"]);
    }

    #[test]
    fn paddle_ocr_success_response_becomes_text_lines() {
        let response = r#"{
            "code": 100,
            "data": [
                {"text": "第一行", "score": 0.99, "box": [[0,0],[1,0],[1,1],[0,1]]},
                {"text": "第二行", "score": 0.98, "box": [[0,2],[1,2],[1,3],[0,3]]}
            ]
        }"#;

        assert_eq!(
            paddle_ocr_response_to_text(response).unwrap(),
            "第一行\n第二行"
        );
    }

    #[test]
    fn paddle_ocr_no_text_response_becomes_empty_text() {
        let response = r#"{"code": 101, "data": "No text found in image."}"#;

        assert_eq!(paddle_ocr_response_to_text(response).unwrap(), "");
    }

    #[test]
    fn paddle_ocr_error_response_returns_message() {
        let response = r#"{"code": 200, "data": "Image path dose not exist."}"#;

        assert_eq!(
            paddle_ocr_response_to_text(response).unwrap_err(),
            "PaddleOCR-json 识别失败(code=200): Image path dose not exist."
        );
    }
}
