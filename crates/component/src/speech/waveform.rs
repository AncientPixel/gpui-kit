use gpui::{
    App, Entity, IntoElement, ParentElement as _, Pixels, RenderOnce, Styled as _, Window, div, px,
};

use crate::{ActiveTheme as _, Sizable, Size, h_flex};

use super::{SpeechState, state::LEVEL_HISTORY};

/// A live bar graph of a [`SpeechState`]'s recent input levels.
///
/// The newest level is on the trailing end. The bars sit flat and muted while
/// no audio is captured.
#[derive(IntoElement)]
pub struct SpeechWaveform {
    state: Entity<SpeechState>,
    bars: usize,
    size: Size,
}

impl SpeechWaveform {
    /// A waveform for `state`.
    pub fn new(state: &Entity<SpeechState>) -> Self {
        Self {
            state: state.clone(),
            bars: 24,
            size: Size::default(),
        }
    }

    /// Set the number of bars, default 24, at most 48.
    pub fn bars(mut self, bars: usize) -> Self {
        self.bars = bars.clamp(1, LEVEL_HISTORY);
        self
    }

    fn height(&self) -> Pixels {
        match self.size {
            Size::Size(height) => height,
            Size::XSmall => px(12.),
            Size::Small => px(16.),
            Size::Medium => px(20.),
            Size::Large => px(24.),
        }
    }
}

impl Sizable for SpeechWaveform {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for SpeechWaveform {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let height = self.height();
        let bar_width = px(2.);
        let state = self.state.read(cx);
        let color = if state.status().is_capturing() {
            cx.theme().primary
        } else {
            cx.theme().muted_foreground
        };
        let levels = state.levels();
        // Right-align the history: pad the leading bars when fewer levels have
        // arrived than there are bars.
        let skip = levels.len().saturating_sub(self.bars);
        let padding = self.bars.saturating_sub(levels.len());
        let levels = std::iter::repeat_n(0., padding).chain(levels.skip(skip));

        h_flex()
            .h(height)
            .gap(bar_width)
            .items_center()
            .children(levels.map(|level| {
                div()
                    .w(bar_width)
                    .h((height * level).max(bar_width))
                    .rounded_full()
                    .bg(color)
            }))
    }
}
