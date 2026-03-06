use std::any::TypeId;
use crate::annotator::{AnnotatorState, SharedAnnotatorState};
use crate::application::Application;
use crate::global::{ReadGlobal, ReadGlobalMut};
use crate::view::xdg_popup_view::TriggerType;
use crate::view::{AppView, View};
use crate::window::AppWindow;
use egui::{Color32, Frame, Id, Response, Ui, Widget, vec2};
use std::ops::Add;
use std::sync::Arc;
use wayland_protocols::xdg::shell::client::xdg_positioner;

pub type BuildDropdownButtonFn = dyn Fn(&Id, &mut Ui) -> Response;
pub type BuildDropdownAreaFn = dyn Fn(
    &mut Application,
    &mut AppWindow,
    &mut dyn View,
    &mut AnnotatorState,
    &mut Ui,
);

pub struct DropdownBox<'a, 'w, 's, 'v> {
    pub id: Id,
    pub app: &'a mut Application,
    pub window: &'w mut AppWindow,
    pub current_view: &'v mut dyn View,
    pub annotator_state: &'s AnnotatorState,
    pub build_drop_down_box_button_fn: Arc<Box<BuildDropdownButtonFn>>,
    pub build_drop_down_area_fn: Arc<Box<BuildDropdownAreaFn>>,
}

impl Widget for DropdownBox<'_, '_, '_, '_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let build_button_fn = self.build_drop_down_box_button_fn;
        let response = build_button_fn(&self.id, ui);
        if response.clicked() {
            let build_drop_down_area_fn = self.build_drop_down_area_fn;
            let popup_view_id = format!("drop-down-box-area-{}", self.id.value());

            let positioner = self.window.create_positioner(&self.app.global_state);

            let current_view_position = self.current_view.position().unwrap();

            let button_rect = &response.rect;

            // 弹出框的尺寸
            positioner.set_size(
                button_rect.width().round() as i32,
                button_rect.height().round() as i32,
            );

            // 按钮的左上角在窗口中的位置
            let button_top_left = button_rect.left_top().add(vec2(
                current_view_position.x as f32,
                current_view_position.y as f32,
            ));

            // 父表面内的锚点矩形
            positioner.set_anchor_rect(
                button_top_left.x.round() as i32,
                button_top_left.y.round() as i32 + button_rect.height() as i32,
                button_rect.width().round() as i32,
                button_rect.height().round() as i32,
            );

            // 指定锚定矩形的哪一条边或角与弹出窗口对齐
            positioner.set_anchor(xdg_positioner::Anchor::BottomLeft);
            // 弹窗相对于锚点的伸展方向
            positioner.set_gravity(xdg_positioner::Gravity::BottomRight);
            positioner.set_offset(0, -20);
            // 空间不足时的自动调整策略
            positioner.set_constraint_adjustment(xdg_positioner::ConstraintAdjustment::all());
            self.window.create_xdg_popup_view(
                popup_view_id.into(),
                &self.app.global_state,
                TriggerType::MousePress,
                positioner,
                Box::new(move |input, egui_ctx, app, window, current_view| {
                    // 构建 UI 的具体内容
                    egui_ctx.run(input, |ctx| {
                        egui::CentralPanel::default()
                            .frame(Frame::new().fill(Color32::from_hex("#393b40").unwrap()))
                            .show(ctx, |ui| {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                                ui.spacing_mut().item_spacing = vec2(1.0, 0.0);

                                let annotator_state = window
                                    .window_context
                                    .globals_by_type
                                    .take::<SharedAnnotatorState>()
                                    .unwrap();

                                build_drop_down_area_fn(
                                    app,
                                    window,
                                    current_view,
                                    &mut annotator_state.borrow_mut(),
                                    ui,
                                );
                                
                                window.window_context.globals_by_type.insert(TypeId::of::<SharedAnnotatorState>(), annotator_state);
                            });
                    })
                }),
            );
        }
        response
    }
}
