use tray_icon::Icon;

pub fn tray_icon() -> Icon {
    const SIZE: i32 = 32;
    let rgba = (0..SIZE * SIZE)
        .flat_map(|i| {
            // Doubled coordinates put the centre of the even-sized grid on an integer.
            let (x, y) = (2 * (i % SIZE) + 1 - SIZE, 2 * (i / SIZE) + 1 - SIZE);
            let alpha = if x * x + y * y <= SIZE * SIZE {
                0xff
            } else {
                0
            };
            [0x2e, 0xa0, 0x43, alpha]
        })
        .collect();
    Icon::from_rgba(rgba, SIZE as u32, SIZE as u32).expect("icon is 32x32 RGBA")
}

#[cfg(test)]
mod tests {
    #[test]
    fn icon_is_valid_rgba() {
        super::tray_icon();
    }
}
