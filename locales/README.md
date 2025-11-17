# 多语言支持

## 添加新语言

只需 3 步，无需修改代码：

### 1. 创建翻译文件

复制现有的语言文件作为模板：

```bash
cp locales/en.yml locales/ja.yml
```

然后翻译 `ja.yml` 中的内容。

### 2. 更新语言索引

编辑 `locales/languages.json`，添加新语言：

```json
{
  "languages": [
    {
      "code": "zh-CN",
      "name": "简体中文",
      "native_name": "简体中文",
      "file": "zh-CN.yml"
    },
    {
      "code": "en",
      "name": "English",
      "native_name": "English",
      "file": "en.yml"
    },
    {
      "code": "ja",
      "name": "Japanese",
      "native_name": "日本語",
      "file": "ja.yml"
    }
  ],
  "default": "en"
}
```

### 3. 重新编译

```bash
cargo build --release
```

完成！新语言会自动出现在语言选择器中。

## 字段说明

- `code`: 语言代码（ISO 639-1，如 `en`, `zh-CN`, `ja`）
- `name`: 英文名称（用于文档）
- `native_name`: 本地语言名称（显示在 UI 中）
- `file`: 翻译文件名

## 语言选择优先级

程序按以下优先级选择语言：

1. **用户偏好** - 用户手动选择的语言（保存在 `Profiles/language.json`）
2. **系统语言** - 自动检测操作系统语言
3. **默认语言** - `languages.json` 中 `default` 字段指定的语言

### 系统语言匹配规则

- 精确匹配：`zh-CN` → `zh-CN`
- 前缀匹配：`zh-TW` → `zh-CN`（如果没有 `zh-TW`）
- 降级到默认语言：如果都不匹配

### 语言偏好持久化

用户在 UI 中选择语言后，会自动保存到 `Profiles/language.json`，下次启动时会记住用户的选择。

## 翻译文件格式

使用 YAML 格式，支持嵌套和参数：

```yaml
_version: 1

main:
  profile: "Profile:"
  launch: "Launch Game"

status:
  save_failed: "Save failed: %{error}"
```

参数使用 `%{name}` 格式。

## 已支持的语言

- 🇨🇳 简体中文 (zh-CN)
- 🇺🇸 English (en)

欢迎贡献更多语言翻译！
