use std::fs;
use std::thread::spawn;
use crate::annotator::{AnnotatorState, SharedAnnotatorState};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::view::ViewId;
use crate::view::xdg_popup_view::TriggerType;
use crate::window::AppWindow;
use egui::{
    Align, Align2, Button, Color32, FontId, Frame, Margin, Pos2, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, Vec2, Widget, pos2, vec2,
};
use rfd::FileDialog;
use wayland_protocols::xdg::shell::client::xdg_positioner;
use crate::clipboard::to_png_bytes;
use crate::context::Command;
use crate::global::{ReadGlobal, ReadGlobalMut};

pub fn create_context_menu(
    view_id: ViewId,
    app: &mut Application,
    window: &mut AppWindow,
    pointer_position: Pos2,
) {
    let positioner = window.create_positioner(&app.global_state);

    let logical_size = LogicalSize::new(160, 160);

    // 弹出框的尺寸
    positioner.set_size(logical_size.width, logical_size.height);

    // 父表面内的锚点矩形
    positioner.set_anchor_rect(
        pointer_position.x.round() as i32,
        pointer_position.y.round() as i32,
        2,
        2,
    );

    // 指定锚定矩形的哪一条边或角与弹出窗口对齐
    positioner.set_anchor(xdg_positioner::Anchor::TopLeft);
    // 弹窗相对于锚点的伸展方向
    positioner.set_gravity(xdg_positioner::Gravity::BottomRight);
    // 空间不足时的自动调整策略
    positioner.set_constraint_adjustment(xdg_positioner::ConstraintAdjustment::all());
    window.create_xdg_popup_view(
        view_id,
        &app.global_state,
        TriggerType::MousePress,
        positioner,
        Box::new(|input, egui_ctx, app, window, current_view| {
            // 构建 UI 的具体内容
            egui_ctx.run(input, move |ctx| {
                let size = current_view.size();
                let margin = Margin::same(15);
                ctx.style_mut(|style| {
                    style.visuals.widgets.inactive.bg_fill = Color32::from_hex("#36363a").unwrap();
                    style.visuals.widgets.hovered.bg_fill = Color32::from_hex("#3077cd").unwrap();
                    style.visuals.widgets.active.bg_fill = Color32::from_hex("#3077cd").unwrap();
                });
                egui::CentralPanel::default()
                    .frame(
                        Frame::new()
                            .fill(ctx.style().visuals.widgets.inactive.bg_fill)
                            .inner_margin(margin)
                            .corner_radius(12.),
                    )
                    .show(ctx, |ui| {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                        ui.vertical_centered(|ui| {
                            let menu_item_size = vec2(size.width as f32 - margin.sum().x, 28.0);

                            let mut primary_toolbar_visible = false;
                            if window
                                .views
                                .contains_key(&AnnotatorState::primary_toolbar_id())
                            {
                                if let Some(Some(view)) =
                                    window.views.get(&AnnotatorState::primary_toolbar_id())
                                {
                                    primary_toolbar_visible = view.get_view_ref().visible();
                                }
                            }

                            let annotator_state = window.window_context.globals_by_type.require_ref::<SharedAnnotatorState>();

                            if ui
                                .add(MenuItem::checkable(
                                    "显示工具条",
                                    menu_item_size,
                                    primary_toolbar_visible,
                                ))
                                .clicked()
                            {
                                if primary_toolbar_visible {
                                    annotator_state.borrow_mut().deactivate_annotation_tool();
                                    window.window_context.commands.push_back(Command::HideView(AnnotatorState::primary_toolbar_id()));
                                }else {
                                    if let Some(Some(view)) = window.views.get_mut(&AnnotatorState::primary_toolbar_id()) {
                                        view.get_view_ref_mut().set_visible(true);
                                    }
                                }
                                current_view.close_later();
                            }
                            if ui
                                .add(MenuItem::new("复制到剪切板", menu_item_size))
                                .clicked()
                            {
                                let image = annotator_state.borrow_mut().take_screenshot(ui.pixels_per_point());
                                window.window_context.commands.push_back(Command::CopyImage(image));
                                current_view.close_later();
                            }
                            if ui
                                .add(MenuItem::new("导出到文件", menu_item_size))
                                .clicked()
                            {
                                let image_receiver = annotator_state.borrow_mut().take_screenshot(ui.pixels_per_point());;
                                spawn(move ||{
                                    let image = image_receiver.recv().unwrap();
                                    let image = to_png_bytes(&*image).unwrap();
                                    if let Some(mut file_path) = FileDialog::new().save_file() {
                                        if file_path.extension().is_none() {
                                            file_path.set_extension("png");
                                        }
                                        fs::write(file_path, image).unwrap();
                                    }
                                });
                            }
                            if ui.add(MenuItem::new("关闭窗口", menu_item_size)).clicked() {}
                        });
                    });
            })
        }),
    );
}

struct MenuItem {
    label: &'static str,
    size: Vec2,
    checkable: bool,
    checked: bool,
}

impl MenuItem {
    fn new(label: &'static str, size: Vec2) -> Self {
        Self {
            label,
            size,
            checkable: false,
            checked: false,
        }
    }

    fn checkable(label: &'static str, size: Vec2, checked: bool) -> Self {
        Self {
            label,
            size,
            checkable: true,
            checked,
        }
    }
}

impl Widget for MenuItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());
        let style = ui.style().clone();
        let painter = ui.painter();
        if response.clicked() {
            painter.rect_filled(rect, 6., style.visuals.widgets.active.bg_fill);
        } else if response.hovered() {
            painter.rect_filled(rect, 6., style.visuals.widgets.hovered.bg_fill);
        } else {
            painter.rect_filled(rect, 6., style.visuals.widgets.inactive.bg_fill);
        }
        let checkbox_margin_left = 5.;
        let checkbox_size = vec2(14., 14.);
        let text_margin_left = checkbox_margin_left + checkbox_size.x + 7.;
        let text_pos = rect.center() - vec2(self.size.x / 2. - text_margin_left, 0.0);
        painter.text(
            text_pos,
            Align2::LEFT_CENTER,
            self.label,
            FontId::proportional(14.),
            Color32::WHITE,
        );

        if self.checkable {
            let checkbox_rect = Rect::from_min_size(
                pos2(
                    rect.left() + checkbox_margin_left,
                    rect.center().y - checkbox_size.y / 2.,
                ),
                checkbox_size,
            );

            if self.checked {
                painter.rect_filled(checkbox_rect, 3., style.visuals.widgets.hovered.bg_fill);
                // 绘制一个简单的勾：两个线段组成
                let left = Pos2::new(checkbox_rect.min.x + 4.0, checkbox_rect.center().y);
                let bottom = Pos2::new(checkbox_rect.center().x, checkbox_rect.max.y - 4.0);
                let right = Pos2::new(checkbox_rect.max.x - 4.0, checkbox_rect.min.y + 4.0);
                painter.line_segment([left, bottom], Stroke::new(2.0, Color32::WHITE));
                painter.line_segment([bottom, right], Stroke::new(2.0, Color32::WHITE));
            } else {
                painter.rect(
                    checkbox_rect,
                    3.,
                    Color32::from_hex("#515154").unwrap(),
                    Stroke::new(1., Color32::from_hex("#2f2f33").unwrap()),
                    StrokeKind::Middle,
                );
            }
        }

        response
    }
}
