//! Motion catalog adapted from zeron's GPUI motion kit.
//!
//! Entrances use an expo-out curve with a small translateY; menus and dialogs
//! use a quicker ease with a 2px settle. All helpers take a `reduce_motion`
//! flag: reduced motion keeps an opacity fade and drops the positional move.
//! UI durations stay under 300ms.

use std::time::Duration;

use gpui::{Animation, AnimationExt, App, ElementId, Global, IntoElement, Styled, px};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezier {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    fn coefficients(a: f32, b: f32) -> (f32, f32, f32) {
        let c = 3.0 * a;
        let bb = 3.0 * (b - a) - c;
        let aa = 1.0 - c - bb;
        (aa, bb, c)
    }

    fn sample_x(&self, t: f32) -> f32 {
        let (a, b, c) = Self::coefficients(self.x1, self.x2);
        ((a * t + b) * t + c) * t
    }

    fn sample_y(&self, t: f32) -> f32 {
        let (a, b, c) = Self::coefficients(self.y1, self.y2);
        ((a * t + b) * t + c) * t
    }

    fn sample_x_derivative(&self, t: f32) -> f32 {
        let (a, b, c) = Self::coefficients(self.x1, self.x2);
        (3.0 * a * t + 2.0 * b) * t + c
    }

    fn solve_t_for_x(&self, x: f32) -> f32 {
        let mut t = x;
        for _ in 0..8 {
            let error = self.sample_x(t) - x;
            if error.abs() < 1e-6 {
                return t;
            }
            let derivative = self.sample_x_derivative(t);
            if derivative.abs() < 1e-6 {
                break;
            }
            t -= error / derivative;
        }
        let (mut low, mut high) = (0.0_f32, 1.0_f32);
        for _ in 0..32 {
            let middle = (low + high) / 2.0;
            if self.sample_x(middle) < x {
                low = middle;
            } else {
                high = middle;
            }
        }
        (low + high) / 2.0
    }

    pub fn eval(&self, x: f32) -> f32 {
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        self.sample_y(self.solve_t_for_x(x)).clamp(0.0, 1.0)
    }
}

/// CSS `cubic-bezier(0.16, 1, 0.3, 1)` — entrances.
pub const EASE_OUT_EXPO: CubicBezier = CubicBezier::new(0.16, 1.0, 0.3, 1.0);
/// CSS `ease` — quick fades, menus, dialogs.
pub const EASE: CubicBezier = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
/// CSS `ease-out` — slides and panels.
pub const EASE_OUT: CubicBezier = CubicBezier::new(0.0, 0.0, 0.58, 1.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionSpec {
    pub duration_ms: u64,
    pub delay_ms: u64,
    pub curve: CubicBezier,
}

impl MotionSpec {
    pub const fn new(duration_ms: u64, curve: CubicBezier) -> Self {
        Self {
            duration_ms,
            delay_ms: 0,
            curve,
        }
    }

    pub const fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub fn total(&self) -> Duration {
        Duration::from_millis(self.delay_ms + self.duration_ms)
    }

    pub fn progress(&self, raw_delta: f32) -> f32 {
        let total = (self.delay_ms + self.duration_ms) as f32;
        if total <= 0.0 || self.duration_ms == 0 {
            return 1.0;
        }
        let t =
            (raw_delta.clamp(0.0, 1.0) * total - self.delay_ms as f32) / self.duration_ms as f32;
        self.curve.eval(t.clamp(0.0, 1.0))
    }

    pub fn animation(&self) -> Animation {
        let spec = *self;
        Animation::new(spec.total()).with_easing(move |delta| spec.progress(delta))
    }
}

/// Page/detail entrance: 220ms expo-out fade + 8px rise.
pub const PAGE_IN: MotionSpec = MotionSpec::new(220, EASE_OUT_EXPO);
/// Dialog entrance: 230ms fade, rising from 14px below.
pub const DIALOG_IN: MotionSpec = MotionSpec::new(230, EASE_OUT_EXPO);
/// Backdrop fade.
pub const BACKDROP_IN: MotionSpec = MotionSpec::new(160, EASE_OUT);
/// Tab/segmented view switch.
pub const TAB_IN: MotionSpec = MotionSpec::new(160, EASE_OUT_EXPO);
/// Popover/menu entrance: 140ms fade + 2px rise.
pub const MENU_IN: MotionSpec = MotionSpec::new(140, EASE);
/// Toast entrance: 180ms fade + 8px rise.
pub const TOAST_IN: MotionSpec = MotionSpec::new(180, EASE_OUT);
/// Expandable panel entrance: 180ms fade + 6px lift.
pub const PANEL_IN: MotionSpec = MotionSpec::new(180, EASE_OUT);
/// Table/list row entrance: 160ms fade + 3px rise.
pub const ROW_IN: MotionSpec = MotionSpec::new(160, EASE_OUT_EXPO);
/// Reduced-motion fallback: 150ms opacity only.
pub const FADE_QUICK: MotionSpec = MotionSpec::new(150, EASE);

#[derive(Clone, Copy, Debug, Default)]
pub struct MotionPreference {
    pub reduce_motion: bool,
}

impl Global for MotionPreference {}

pub fn reduce_motion(cx: &App) -> bool {
    cx.try_global::<MotionPreference>()
        .map(|preference| preference.reduce_motion)
        .unwrap_or(false)
}

pub fn page_in<E>(id: impl Into<ElementId>, element: E, reduce: bool) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    if reduce {
        element
            .with_animation(id, FADE_QUICK.animation(), |element, t| element.opacity(t))
            .into_any_element()
    } else {
        element
            .with_animation(id, PAGE_IN.animation(), |element, t| {
                element.relative().opacity(t).top(px(8.0 * (1.0 - t)))
            })
            .into_any_element()
    }
}

pub fn backdrop_in<E>(id: impl Into<ElementId>, element: E, reduce: bool) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    let spec = if reduce { FADE_QUICK } else { BACKDROP_IN };
    element
        .with_animation(id, spec.animation(), |element, t| element.opacity(t))
        .into_any_element()
}

/// A small fade/rise for tab and segmented-control content changes.
pub fn tab_in<E>(id: impl Into<ElementId>, element: E, reduce: bool) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    if reduce {
        element
            .with_animation(id, FADE_QUICK.animation(), |element, t| element.opacity(t))
            .into_any_element()
    } else {
        element
            .with_animation(id, TAB_IN.animation(), |element, t| {
                element.relative().opacity(t).top(px(5.0 * (1.0 - t)))
            })
            .into_any_element()
    }
}

pub fn dialog_in<E>(id: impl Into<ElementId>, element: E, reduce: bool) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    if reduce {
        element
            .with_animation(id, FADE_QUICK.animation(), |element, t| element.opacity(t))
            .into_any_element()
    } else {
        element
            .with_animation(id, DIALOG_IN.animation(), |element, t| {
                element.relative().opacity(t).top(px(14.0 * (1.0 - t)))
            })
            .into_any_element()
    }
}

pub fn menu_in<E>(id: impl Into<ElementId>, element: E, reduce: bool) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    if reduce {
        element
            .with_animation(id, FADE_QUICK.animation(), |element, t| element.opacity(t))
            .into_any_element()
    } else {
        element
            .with_animation(id, MENU_IN.animation(), |element, t| {
                element
                    .relative()
                    .opacity(0.3 + 0.7 * t)
                    .top(px(-2.0 * (1.0 - t)))
            })
            .into_any_element()
    }
}

pub fn toast_in<E>(id: impl Into<ElementId>, element: E, reduce: bool) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    if reduce {
        element
            .with_animation(id, FADE_QUICK.animation(), |element, t| element.opacity(t))
            .into_any_element()
    } else {
        element
            .with_animation(id, TOAST_IN.animation(), |element, t| {
                element.relative().opacity(t).top(px(8.0 * (1.0 - t)))
            })
            .into_any_element()
    }
}

pub fn panel_in<E>(id: impl Into<ElementId>, element: E, reduce: bool) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    if reduce {
        element
            .with_animation(id, FADE_QUICK.animation(), |element, t| element.opacity(t))
            .into_any_element()
    } else {
        element
            .with_animation(id, PANEL_IN.animation(), |element, t| {
                element.relative().opacity(t).top(px(-6.0 * (1.0 - t)))
            })
            .into_any_element()
    }
}

pub fn row_in<E>(
    id: impl Into<ElementId>,
    index: usize,
    element: E,
    reduce: bool,
) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    if reduce {
        return element
            .with_animation(id, FADE_QUICK.animation(), |element, t| element.opacity(t))
            .into_any_element();
    }
    let delay = (index.min(4) as u64) * 30;
    element
        .with_animation(id, ROW_IN.with_delay(delay).animation(), |element, t| {
            element.relative().opacity(t).top(px(3.0 * (1.0 - t)))
        })
        .into_any_element()
}

pub fn fade_quick<E>(id: impl Into<ElementId>, element: E) -> impl IntoElement
where
    E: Styled + IntoElement + 'static,
{
    element
        .with_animation(id, FADE_QUICK.animation(), |element, t| element.opacity(t))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_are_clamped_and_monotonic() {
        for curve in [EASE_OUT_EXPO, EASE, EASE_OUT] {
            assert_eq!(curve.eval(0.0), 0.0);
            assert_eq!(curve.eval(1.0), 1.0);
            let mut previous = 0.0;
            for step in 0..=20 {
                let value = curve.eval(step as f32 / 20.0);
                assert!(value >= previous - 1e-4);
                assert!((0.0..=1.0).contains(&value));
                previous = value;
            }
        }
    }

    #[test]
    fn delayed_specs_hold_zero_then_finish() {
        let spec = ROW_IN.with_delay(50);
        assert_eq!(spec.progress(0.0), 0.0);
        let delay_fraction = spec.delay_ms as f32 / spec.total().as_millis() as f32;
        assert_eq!(spec.progress(delay_fraction * 0.5), 0.0);
        assert_eq!(spec.progress(1.0), 1.0);
        let halfway = spec.progress(0.5);
        assert!((0.0..=1.0).contains(&halfway));
    }
}
