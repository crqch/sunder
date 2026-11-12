use crate::assets::{Icon, icon};
use crate::format;
use crate::theme::{Theme, text as type_scale};
use gpui::{
    Animation, AnimationExt, AnyView, Div, Hsla, IntoElement, Render, SharedString, Stateful,
    Transformation, Window, div, percentage, prelude::*, px,
};
use std::time::Duration;

pub fn page_title(
    theme: &Theme,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(5.0))
        .child(
            div()
                .text_size(px(type_scale::TITLE))
                .line_height(px(31.0))
                .font_weight(type_scale::SEMIBOLD)
                .text_color(theme.foreground)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(type_scale::BODY))
                .text_color(theme.muted)
                .child(subtitle.into()),
        )
}

pub fn section_title(theme: &Theme, title: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(type_scale::HEADING))
        .line_height(px(22.0))
        .font_weight(type_scale::SEMIBOLD)
        .text_color(theme.foreground)
        .child(title.into())
}

pub fn card(theme: &Theme) -> Div {
    div()
        .bg(theme.surface)
        .border_1()
        .border_color(theme.border)
        .rounded_xl()
        .shadow(vec![gpui::BoxShadow {
            color: theme.shadow,
            offset: gpui::point(px(0.0), px(3.0)),
            blur_radius: px(14.0),
            spread_radius: px(0.0),
        }])
}

pub fn primary_button(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    icon: Option<Icon>,
) -> Stateful<Div> {
    let hover = theme.primary_hover;
    let pressed = theme.primary_pressed;
    let mut button = div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(7.0))
        .h(px(38.0))
        .px(px(14.0))
        .rounded_lg()
        .bg(theme.primary)
        .text_color(theme.on_primary)
        .text_size(px(type_scale::BODY))
        .font_weight(type_scale::MEDIUM)
        .relative()
        .cursor_pointer()
        .shadow(vec![gpui::BoxShadow {
            color: theme.primary.opacity(0.22),
            offset: gpui::point(px(0.0), px(2.0)),
            blur_radius: px(8.0),
            spread_radius: px(0.0),
        }])
        .hover(move |style| style.bg(hover))
        .active(move |style| style.bg(pressed).opacity(0.9));

    if let Some(icon) = icon {
        button = button.child(icon_el(icon, 15.0, theme.on_primary));
    }
    button.child(label.into())
}

pub fn secondary_button(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    icon: Option<Icon>,
) -> Stateful<Div> {
    let hover = theme.surface_hover;
    let hover_border = theme.border_strong;
    let mut button = div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(7.0))
        .h(px(38.0))
        .px(px(13.0))
        .rounded_lg()
        .bg(theme.surface)
        .border_1()
        .border_color(theme.border)
        .text_color(theme.foreground)
        .text_size(px(type_scale::BODY))
        .font_weight(type_scale::MEDIUM)
        .relative()
        .cursor_pointer()
        .hover(move |style| style.bg(hover).border_color(hover_border))
        .active(|style| style.opacity(0.9));

    if let Some(icon) = icon {
        button = button.child(icon_el(icon, 15.0, theme.muted));
    }
    button.child(label.into())
}

pub fn danger_button(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    icon: Option<Icon>,
) -> Stateful<Div> {
    let hover = theme.expense.opacity(0.9);
    let mut button = div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(7.0))
        .h(px(38.0))
        .px(px(14.0))
        .rounded_lg()
        .bg(theme.expense)
        .text_color(theme.on_primary)
        .text_size(px(type_scale::BODY))
        .font_weight(type_scale::MEDIUM)
        .relative()
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .active(|style| style.opacity(0.9));

    if let Some(icon) = icon {
        button = button.child(icon_el(icon, 15.0, theme.on_primary));
    }
    button.child(label.into())
}

pub fn ghost_button(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    icon: Option<Icon>,
) -> Stateful<Div> {
    let hover = theme.surface_muted;
    let hover_text = theme.foreground;
    let mut button = div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .h(px(32.0))
        .px(px(10.0))
        .rounded_md()
        .text_color(theme.muted)
        .text_size(px(type_scale::SMALL))
        .font_weight(type_scale::MEDIUM)
        .relative()
        .cursor_pointer()
        .hover(move |style| style.bg(hover).text_color(hover_text))
        .active(|style| style.opacity(0.9));

    if let Some(icon) = icon {
        button = button.child(icon_el(icon, 14.0, theme.muted));
    }
    button.child(label.into())
}

pub fn danger_ghost_button(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    icon: Option<Icon>,
) -> Stateful<Div> {
    let hover_bg = theme.expense_tint;
    let color = theme.expense;
    let mut button = div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .h(px(32.0))
        .px(px(10.0))
        .rounded_md()
        .text_color(color)
        .text_size(px(type_scale::SMALL))
        .font_weight(type_scale::MEDIUM)
        .relative()
        .cursor_pointer()
        .hover(move |style| style.bg(hover_bg).text_color(color))
        .active(|style| style.opacity(0.9));

    if let Some(icon) = icon {
        button = button.child(icon_el(icon, 14.0, color));
    }
    button.child(label.into())
}

pub fn icon_button(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    name: Icon,
    small: bool,
) -> Stateful<Div> {
    let size = if small { 30.0 } else { 34.0 };
    let icon_size = if small { 15.0 } else { 16.0 };
    let hover = theme.surface_muted;
    let hover_text = theme.foreground;
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .size(px(size))
        .rounded_md()
        .relative()
        .cursor_pointer()
        .text_color(theme.muted)
        .hover(move |style| style.bg(hover).text_color(hover_text))
        .active(|style| style.opacity(0.9))
        .child(icon_el(name, icon_size, theme.muted))
}

pub fn link_button(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    icon: Option<Icon>,
) -> Stateful<Div> {
    let hover = theme.primary_hover;
    let mut button = div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .text_size(px(type_scale::SMALL))
        .font_weight(type_scale::MEDIUM)
        .text_color(theme.primary)
        .relative()
        .cursor_pointer()
        .hover(move |style| style.text_color(hover))
        .active(|style| style.opacity(0.8));

    button = button.child(label.into());
    if let Some(icon) = icon {
        button = button.child(icon_el(icon, 13.0, theme.primary));
    }
    button
}

pub fn chip(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    dot: Option<Hsla>,
) -> Stateful<Div> {
    let hover = theme.surface_hover;
    let hover_text = theme.foreground;
    let mut element = div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .h(px(30.0))
        .px(px(11.0))
        .rounded_md()
        .border_1()
        .text_size(px(type_scale::SMALL))
        .font_weight(type_scale::MEDIUM)
        .relative()
        .cursor_pointer()
        .active(|style| style.opacity(0.88))
        .when(selected, |style| {
            style
                .bg(theme.primary_tint)
                .border_color(theme.primary.opacity(0.35))
                .text_color(theme.primary)
        })
        .when(!selected, |style| {
            style
                .bg(theme.surface)
                .border_color(theme.border)
                .text_color(theme.muted)
                .hover(move |style| style.bg(hover).text_color(hover_text))
        });

    if let Some(dot) = dot {
        element = element.child(div().size(px(7.0)).rounded_full().bg(dot).flex_shrink_0());
    }
    element.child(label.into())
}

pub fn badge(theme: &Theme, label: impl Into<SharedString>, kind: BadgeKind) -> Div {
    let (fg, bg) = match kind {
        BadgeKind::Neutral => (theme.muted, theme.surface_muted),
        BadgeKind::Primary => (theme.primary, theme.primary_tint),
        BadgeKind::Success => (theme.income, theme.income_tint),
        BadgeKind::Danger => (theme.expense, theme.expense_tint),
        BadgeKind::Warning => (theme.warning, theme.warning.opacity(0.12)),
    };
    div()
        .flex()
        .items_center()
        .gap(px(4.0))
        .h(px(22.0))
        .px(px(7.0))
        .rounded_md()
        .bg(bg)
        .text_color(fg)
        .text_size(px(type_scale::MICRO))
        .font_weight(type_scale::SEMIBOLD)
        .child(label.into())
}

#[derive(Clone, Copy)]
pub enum BadgeKind {
    Neutral,
    Primary,
    Success,
    Danger,
    Warning,
}

pub fn dot(color: Hsla, size: f32) -> Div {
    div()
        .size(px(size))
        .rounded_full()
        .bg(color)
        .flex_shrink_0()
}

pub fn amount(
    theme: &Theme,
    value: f64,
    symbol: &str,
    position: crate::format::CurrencyPosition,
    size: f32,
) -> Div {
    div()
        .text_size(px(size))
        .font_weight(type_scale::SEMIBOLD)
        .text_color(theme.money_color(value))
        .child(format::signed_money(value, symbol, position))
}

pub fn keycap(theme: &Theme, label: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .min_w(px(18.0))
        .h(px(18.0))
        .px(px(4.0))
        .rounded_sm()
        .bg(theme.surface_muted)
        .border_1()
        .border_color(theme.border)
        .text_size(px(9.5))
        .font_weight(type_scale::MEDIUM)
        .text_color(theme.subtle)
        .child(label.into())
}

pub fn shortcut(theme: &Theme, keys: &[&str]) -> Div {
    div().flex().items_center().gap(px(3.0)).children(
        keys.iter()
            .map(|key| keycap(theme, key.to_string()).into_any_element()),
    )
}

pub fn empty_state(
    theme: &Theme,
    name: Icon,
    title: impl Into<SharedString>,
    message: impl Into<SharedString>,
) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .p(px(40.0))
        .w_full()
        .rounded_xl()
        .border_1()
        .border_dashed()
        .border_color(theme.border)
        .bg(theme.surface.opacity(0.6))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(46.0))
                .rounded_xl()
                .bg(theme.primary_tint)
                .child(icon_el(name, 20.0, theme.primary)),
        )
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(type_scale::SEMIBOLD)
                .text_color(theme.foreground)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(type_scale::SMALL))
                .text_color(theme.muted)
                .text_center()
                .max_w(px(330.0))
                .child(message.into()),
        )
}

pub fn divider(theme: &Theme) -> Div {
    div().h(px(1.0)).w_full().bg(theme.border)
}

pub fn spinner(theme: &Theme, size: f32) -> impl IntoElement {
    icon(Icon::Refresh, size, theme.muted).with_animation(
        "sunder-spinner",
        Animation::new(Duration::from_millis(750)).repeat(),
        |element, delta| element.with_transformation(Transformation::rotate(percentage(delta))),
    )
}

pub fn selection_checkbox(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    checked: bool,
) -> Stateful<Div> {
    let hover = theme.border_strong;
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .size(px(19.0))
        .rounded_full()
        .border_2()
        .relative()
        .cursor_pointer()
        .active(|style| style.opacity(0.85))
        .when(checked, |style| {
            style.bg(theme.primary).border_color(theme.primary)
        })
        .when(!checked, |style| {
            style
                .bg(theme.surface)
                .border_color(theme.border)
                .hover(move |style| style.border_color(hover))
        })
        .child(if checked {
            icon_el(Icon::Check, 11.0, theme.on_primary).into_any_element()
        } else {
            div().into_any_element()
        })
}

pub fn icon_el(name: Icon, size: f32, color: Hsla) -> gpui::Svg {
    icon(name, size, color)
}

struct TooltipView {
    text: SharedString,
    keys: Vec<SharedString>,
}

impl Render for TooltipView {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let palette = crate::theme::theme(cx).clone();
        div()
            .debug_selector(|| "sunder-tooltip".to_string())
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .py(px(5.0))
            .rounded_md()
            .bg(palette.foreground)
            .text_color(palette.bg)
            .text_size(px(10.5))
            .font_weight(type_scale::MEDIUM)
            .font_family(palette.font.clone())
            .whitespace_nowrap()
            .shadow(vec![gpui::BoxShadow {
                color: palette.shadow,
                offset: gpui::point(px(0.0), px(3.0)),
                blur_radius: px(8.0),
                spread_radius: px(0.0),
            }])
            .child(self.text.clone())
            .children(self.keys.iter().map(|key| {
                div()
                    .px(px(4.0))
                    .py(px(1.0))
                    .rounded_sm()
                    .bg(palette.bg.opacity(0.16))
                    .text_color(palette.bg)
                    .text_size(px(9.5))
                    .font_weight(type_scale::SEMIBOLD)
                    .child(key.clone())
            }))
    }
}

pub trait SunderTooltip: StatefulInteractiveElement + Sized {
    fn sunder_tooltip(self, text: impl Into<SharedString>, keys: &[&str]) -> Self {
        let text: SharedString = text.into();
        let keys: Vec<SharedString> = keys
            .iter()
            .map(|key| SharedString::from(key.to_string()))
            .collect();
        self.tooltip(move |_, cx| {
            let view: AnyView = cx
                .new(|_| TooltipView {
                    text: text.clone(),
                    keys: keys.clone(),
                })
                .into();
            view
        })
    }
}

impl<T: StatefulInteractiveElement + Sized> SunderTooltip for T {}

pub fn label(theme: &Theme, value: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(type_scale::SMALL))
        .font_weight(type_scale::MEDIUM)
        .text_color(theme.muted)
        .child(value.into())
}

pub fn field_label(theme: &Theme, value: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(type_scale::SMALL))
        .font_weight(type_scale::MEDIUM)
        .text_color(theme.foreground)
        .child(value.into())
}

pub fn icon_badge(theme: &Theme, name: Icon, tone: BadgeKind) -> Div {
    let (fg, bg) = match tone {
        BadgeKind::Neutral => (theme.muted, theme.surface_muted),
        BadgeKind::Primary => (theme.primary, theme.primary_tint),
        BadgeKind::Success => (theme.income, theme.income_tint),
        BadgeKind::Danger => (theme.expense, theme.expense_tint),
        BadgeKind::Warning => (theme.warning, theme.warning.opacity(0.12)),
    };
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(34.0))
        .rounded_lg()
        .bg(bg)
        .child(icon_el(name, 17.0, fg))
}
