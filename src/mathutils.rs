use sdl2::rect::{ Point, Rect };

/// Returns a rectangular selection box with center
/// and not exceeding a specified point, while keeping
/// width and height ratio.
pub fn selection_from_center_with_ratio<P, Q>(center: P, dragpoint: Q, ratio: f32) -> Rect
    where P: Into<Point>, Q: Into<Point> {
    let center = center.into();
    let vector = dragpoint.into() - center;
    let vector = Point::new(
        vector.x().abs(),
        vector.y().abs()
    );

    if vector.x() as f32 / ratio > vector.y() as f32 {
        Rect::from_center(
            center,
            2 * (vector.y() as f32 * ratio) as u32,
            2 * vector.y() as u32
        )
    } else {
        Rect::from_center(
            center,
            2 * vector.x() as u32,
            2 * (vector.x() as f32 / ratio) as u32
        )
    }
}
