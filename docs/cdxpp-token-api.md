# save_cdxpp_token 接口

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/save_cdxpp_token` |
| 调试端口 | `7655` |
| 发布端口 | `7654` |
| 文件路径 | `E:\chatE\cdxpp.token` |
| 请求体大小 | 最大 `64 KiB` |

完整地址示例：

```text
http://127.0.0.1:7655/save_cdxpp_token
```

## 请求体

请求体必须是 UTF-8 字符串。接口支持以下三种格式。

### JSON 对象（推荐）

```http
POST /save_cdxpp_token HTTP/1.1
Content-Type: application/json

{"str":"your-cdxpp-token"}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `str` | `string` | 是 | 要保存的 cdxpp token |

### JSON 字符串

```http
POST /save_cdxpp_token HTTP/1.1
Content-Type: application/json

"your-cdxpp-token"
```

### 纯文本

```http
POST /save_cdxpp_token HTTP/1.1
Content-Type: text/plain; charset=utf-8

your-cdxpp-token
```

JSON 格式会保存解析后的字符串值；纯文本格式会按请求体原样保存，不会自动去除首尾空白。

## 响应

### 保存成功

HTTP 状态码：`200 OK`

```text
cdxpp token 保存成功
```

### 请求体无效

HTTP 状态码：`400 Bad Request`

可能原因：请求体不是 UTF-8，或 JSON 值不是字符串，也没有包含字符串类型的 `str` 字段。

### 保存失败

HTTP 状态码：`500 Internal Server Error`

可能原因：程序无法创建 `E:\chatE` 目录或无法写入 `cdxpp.token` 文件。

## Windows 调用示例

使用 PowerShell 发送 JSON 对象：

```powershell
$body = @{ str = "your-cdxpp-token" } | ConvertTo-Json -Compress
Invoke-RestMethod `
  -Method Post `
  -Uri "http://127.0.0.1:7655/save_cdxpp_token" `
  -ContentType "application/json; charset=utf-8" `
  -Body $body
```

使用 `curl.exe` 发送纯文本：

```powershell
curl.exe -X POST "http://127.0.0.1:7655/save_cdxpp_token" `
  -H "Content-Type: text/plain; charset=utf-8" `
  --data-raw "your-cdxpp-token"
```

## 保存行为

- `E:\chatE` 不存在时，接口会自动创建目录。
- 文件已存在时会覆盖原内容。
- 文件内容使用 UTF-8 写入。
- 接口允许跨域请求，并允许 `POST` 方法。
