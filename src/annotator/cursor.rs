use egui::{Color32, Painter, Pos2, Rangef, Shape, Stroke};

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

    pub fn paint_with(self, painter: &Painter) {
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
        let line_length = 10f32;

        // 线段靠近圆点的那个端点和圆点圆周之间的距离
        let margin = 7f32;

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
