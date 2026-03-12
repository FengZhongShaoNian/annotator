use egui::{pos2, vec2, Color32, FontId, Painter, Pos2, Rangef, Rect, Shape, Stroke, StrokeKind, Ui};

pub trait CustomCursor {
    fn paint_with(&self, painter: &Painter);
}

/// 中间带圆点的十字光标
pub struct Crosshair {
    /// 光标中点的坐标
    center_pos: Pos2,
    /// 光标的颜色
    color: Color32,
    /// 光标中间的圆点的直径
    crosshair_dot_diameter: f32,
}

impl Crosshair {
    pub fn new(center_pos: Pos2, color: Color32, crosshair_dot_diameter: f32) -> Self {
        Self {
            center_pos,
            color,
            crosshair_dot_diameter,
        }
    }
}

impl CustomCursor for Crosshair {
    fn paint_with(&self, painter: &Painter) {
        let crosshair_dot_radius = self.crosshair_dot_diameter / 2.;
        let center_pos = self.center_pos;
        let color = self.color;

        // 绘制中间的圆点
        painter.circle(
            center_pos,
            crosshair_dot_radius,
            color,
            Stroke::new(0.5, color),
        );

        // 线段的长度
        let line_length = 8f32;

        // 线段靠近圆点的那个端点和圆点圆周之间的距离
        let margin = 5f32;

        let stroke = Stroke::new(2., color);

        // 绘制左侧的线段
        painter.add(Shape::hline(
            Rangef::new(
                center_pos.x - crosshair_dot_radius - margin - line_length,
                center_pos.x - crosshair_dot_radius - margin,
            ),
            center_pos.y,
            stroke,
        ));
        // 绘制右侧的线段
        painter.add(Shape::hline(
            Rangef::new(
                center_pos.x + crosshair_dot_radius + margin,
                center_pos.x + crosshair_dot_radius + margin + line_length,
            ),
            center_pos.y,
            stroke,
        ));

        // 绘制上方的线段
        painter.add(Shape::vline(
            center_pos.x,
            Rangef::new(
                center_pos.y - crosshair_dot_radius - margin - line_length,
                center_pos.y - crosshair_dot_radius - margin,
            ),
            stroke,
        ));
        // 绘制下方的线段
        painter.add(Shape::vline(
            center_pos.x,
            Rangef::new(
                center_pos.y + crosshair_dot_radius + margin,
                center_pos.y + crosshair_dot_radius + margin + line_length,
            ),
            stroke,
        ));
    }
}


/// 圆形光标
pub struct Circle {
    /// 光标中点的坐标
    center_pos: Pos2,
    /// 光标的颜色
    color: Color32,
    /// 光标的直径
    diameter: f32,
}

impl Circle {
    pub fn new(center_pos: Pos2, color: Color32, diameter: f32) -> Self {
        Self {
            center_pos,
            color,
            diameter,
        }
    }
}

impl CustomCursor for Circle {
    fn paint_with(&self, painter: &Painter) {
        let radius = self.diameter / 2.;
        let center_pos = self.center_pos;
        let color = self.color;

        // 绘制中间的圆点
        painter.circle(
            center_pos,
            radius,
            color,
            Stroke::new(0.5, color),
        );
    }
}

/// 十字箭头光标
pub struct Move {
    /// 光标中点的坐标
    center_pos: Pos2,
    /// 光标的颜色
    color: Color32,
    /// 光标的宽高
    size: f32,
}

impl Move {
    pub fn new(center_pos: Pos2, color: Color32, size: f32) -> Self {
        Self {
            center_pos,
            color,
            size,
        }
    }
}

impl CustomCursor for Move {
    fn paint_with(&self, painter: &Painter) {
        let half_size = self.size / 2.;
        let stroke_width = 2.5;
        // 向左的箭头
        painter.arrow(self.center_pos, vec2(-half_size, 0.), Stroke::new(stroke_width, self.color));
        // 向右的箭头
        painter.arrow(self.center_pos, vec2(half_size, 0.), Stroke::new(stroke_width, self.color));
        // 向上的箭头
        painter.arrow(self.center_pos, vec2(0., -half_size), Stroke::new(stroke_width, self.color));
        // 向下的箭头
        painter.arrow(self.center_pos, vec2(0., half_size), Stroke::new(stroke_width, self.color));
    }
}

#[derive(Clone, Debug)]
pub struct SerialNumberStyle {
    /// 文本颜色
    pub text_color: Color32,
    /// 填充颜色
    pub fill_color: Color32,
    /// 是否绘制矩形外框
    pub draw_rect_stroke: bool,
    /// 圆的半径
    pub radius: f32,
}

impl SerialNumberStyle {
    pub fn new(text_color: Color32, fill_color: Color32, draw_rect_stroke: bool) -> Self {
        Self {
            text_color,
            fill_color,
            draw_rect_stroke,
            radius: 16.,
        }
    }
}

impl Default for SerialNumberStyle {
    fn default() -> Self {
        Self::new(Color32::WHITE, Color32::RED, true)
    }
}

/// 带圆圈的数字光标
#[derive(Clone)]
pub struct SerialNumber {
    /// 光标中点的坐标
    center_pos: Pos2,
    /// 数字
    number: u32,
    style: SerialNumberStyle,
}
impl SerialNumber {
    pub fn new(center_pos: Pos2, number: u32, style: SerialNumberStyle) -> Self {
        Self {
            center_pos,
            number,
            style,
        }
    }

    pub fn rect(&self) -> Rect {
        let radius = self.style.radius;
        Rect::from_min_size(self.center_pos - vec2(radius, radius), vec2(radius*2., radius*2.))
    }

    pub fn style(&self) -> &SerialNumberStyle {
        &self.style
    }

    pub fn style_mut(&mut self) -> &mut SerialNumberStyle {
        &mut self.style
    }
}

impl CustomCursor for SerialNumber {
    fn paint_with(&self, painter: &Painter) {
        // 圆的半径
        let radius = 14.0;
        let rect = self.rect();

        // 绘制圆圈（边框）
        if self.style.draw_rect_stroke {
            painter.rect_stroke(
                rect,
                0.,
                Stroke::new(1., Color32::WHITE),
                StrokeKind::Middle,
            );
        }

        // 绘制圆圈
        painter.circle_filled(self.center_pos, radius, self.style.fill_color);

        // 绘制数字文本
        painter.text(
            self.center_pos,
            egui::Align2::CENTER_CENTER,
            self.number.to_string(),
            FontId::proportional(16.0),
            self.style.text_color,
        );
    }
}