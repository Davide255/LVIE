use crate::matrix::Matrix;

pub fn homography<T: Clone>(mat: Matrix<f32>, img: &mut Matrix<T>, filler: T) {
    let mut output: Vec<T> = Vec::new();

    for i in 0..img.get_content().len() {
        let mut pos = Matrix::new(
            vec![
                (i % img.width()/*- img.width() / 2*/) as f32,
                (i / img.width()/*- img.height() / 2*/) as f32,
                1f32,
            ],
            3,
            1,
        );

        // INEFFICIENT, should keep in memory the values for same x-s and y-s
        pos = (mat.clone() * pos).unwrap();
        let v = pos.get_content();

        output.push(
            match img.get_element(
                ((v[0] / v[2]).round()/*+ (img.width() / 2) as f32*/) as usize,
                ((v[1] / v[2]).round()/*+ (img.height() / 2) as f32*/) as usize,
            ) {
                Ok(val) => val,
                Err(_) => filler.clone(),
            },
        )
    }

    img.update_content(output).unwrap();
}
