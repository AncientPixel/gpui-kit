//! A recognizer that needs no service, permission or network: it types a
//! scripted passage while audio arrives, so the whole flow can be tried
//! anywhere, including on Linux.

use gpui_kit::App;
use gpui_kit::component::speech::{RecognitionSession, SpeechError, SpeechRecognizer, SpeechSink};

/// Audio per recognized token: 0.25 s at 16 kHz.
const SAMPLES_PER_TOKEN: usize = 4_000;

pub struct DemoRecognizer {
    script: &'static Script,
}

impl DemoRecognizer {
    /// A recognizer typing the passage for `language`, a BCP 47 tag.
    pub fn new(language: &str) -> Self {
        let script = SCRIPTS
            .iter()
            .find(|script| language.starts_with(script.language))
            .unwrap_or(&SCRIPTS[0]);
        Self { script }
    }
}

impl SpeechRecognizer for DemoRecognizer {
    fn start(
        &self,
        sink: SpeechSink,
        cx: &mut App,
    ) -> Result<Box<dyn RecognitionSession>, SpeechError> {
        // There is nothing to connect to, so audio is consumed at once.
        sink.ready(cx);
        Ok(Box::new(DemoSession {
            sink,
            script: self.script,
            sentence_ix: 0,
            tokens: 0,
            samples: 0,
        }))
    }
}

struct Script {
    language: &'static str,
    /// Put between tokens and between sentences.
    separator: &'static str,
    sentences: &'static [&'static [&'static str]],
}

const SCRIPTS: &[Script] = &[
    Script {
        language: "en",
        separator: " ",
        sentences: &[
            &[
                "Speech", "input", "turns", "what", "you", "say", "into", "text.",
            ],
            &[
                "Stop", "whenever", "you", "like,", "and", "the", "words", "land", "in", "the",
                "note.",
            ],
            &[
                "Any", "speech", "service", "plugs", "in", "through", "one", "trait.",
            ],
        ],
    },
    Script {
        language: "zh",
        separator: "",
        sentences: &[
            &["语音", "输入", "会把", "你说", "的话", "转成", "文字。"],
            &["随时", "停止，", "文字", "就会", "写进", "笔记。"],
            &[
                "任何",
                "识别",
                "服务",
                "都能",
                "通过",
                "一个",
                "接口",
                "接进来。",
            ],
        ],
    },
    Script {
        language: "ja",
        separator: "",
        sentences: &[
            &["話した", "言葉が", "そのまま", "文字に", "なります。"],
            &["止めると", "ノートに", "書き込まれます。"],
        ],
    },
];

struct DemoSession {
    sink: SpeechSink,
    script: &'static Script,
    sentence_ix: usize,
    /// Tokens of the current sentence heard so far.
    tokens: usize,
    /// Samples received since the last token.
    samples: usize,
}

impl DemoSession {
    /// The heard part of the current sentence, with the separator that joins it
    /// to the sentences before.
    fn heard(&self) -> Option<String> {
        let sentence = self.sentence()?;
        if self.tokens == 0 {
            return None;
        }
        let leading = if self.sentence_ix > 0 {
            self.script.separator
        } else {
            ""
        };
        Some(format!(
            "{leading}{}",
            sentence[..self.tokens].join(self.script.separator)
        ))
    }

    fn sentence(&self) -> Option<&'static [&'static str]> {
        let sentences = self.script.sentences;
        sentences.get(self.sentence_ix % sentences.len()).copied()
    }

    fn next_token(&mut self, cx: &mut App) {
        let Some(sentence) = self.sentence() else {
            return;
        };
        self.tokens += 1;
        if self.tokens < sentence.len() {
            if let Some(heard) = self.heard() {
                self.sink.hypothesis(heard, cx);
            }
        } else {
            self.commit(cx);
        }
    }

    fn commit(&mut self, cx: &mut App) {
        if let Some(heard) = self.heard() {
            self.sink.phrase(heard, cx);
        }
        self.sentence_ix += 1;
        self.tokens = 0;
    }
}

impl RecognitionSession for DemoSession {
    fn push_audio(&mut self, samples: &[i16], cx: &mut App) {
        self.samples += samples.len();
        while self.samples >= SAMPLES_PER_TOKEN {
            self.samples -= SAMPLES_PER_TOKEN;
            self.next_token(cx);
        }
    }

    fn finish(&mut self, cx: &mut App) {
        self.commit(cx);
        self.sink.finish(cx);
    }
}
