//! Traditional Chinese the way people in Taiwan write it day to day. The
//! recognizer outputs Simplified Chinese; OpenCC's `s2twp` converts both the
//! characters and the vocabulary (界面 → 介面, 软件 → 軟體). It follows the
//! official 臺, while everyday writing (and every place name people type)
//! uses 台, so that one character is folded back.

use ferrous_opencc::{config::BuiltinConfig, OpenCC};

pub fn taiwan(text: &str) -> Result<String, String> {
    let converter = OpenCC::from_config(BuiltinConfig::S2twp).map_err(|e| e.to_string())?;
    Ok(converter.convert(text).replace('臺', "台"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tw(s: &str) -> String {
        taiwan(s).unwrap()
    }

    #[test]
    fn taiwan_vocabulary_and_everyday_tai() {
        assert_eq!(
            tw("好啊，那我们明天晚上七点在台北车站见"),
            "好啊，那我們明天晚上七點在台北車站見"
        );
        assert_eq!(tw("然后界面的话"), "然後介面的話");
        assert_eq!(
            tw("这个软件的视频功能，默认的内存和网络设置"),
            "這個軟體的影片功能，預設的記憶體和網路設定"
        );
        assert_eq!(tw("台风天去台湾的平台"), "颱風天去台灣的平台");
    }

    #[test]
    fn english_is_untouched() {
        assert_eq!(
            tw("我今天在测试 Moqi 的新版本，repo 在 GitHub"),
            "我今天在測試 Moqi 的新版本，repo 在 GitHub"
        );
    }
}
