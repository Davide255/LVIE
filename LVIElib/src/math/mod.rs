use std::collections::HashMap;

pub fn cumulative_distribution(data: &HashMap<u8, u32>) -> HashMap<u8, u32> {
    let mut output: HashMap<u8, u32> = HashMap::new();
    output.insert(0, *data.get(&0).unwrap());
    for i in 0..255 {
        output.insert(
            i + 1,
            *data.get(&(i + 1)).unwrap() + output.get(&i).unwrap(),
        );
    }

    output
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::math::cumulative_distribution;

    #[test]
    fn cdf_test() {
        let buf: Vec<u8> = vec![
            52, 55, 61, 59, 79, 61, 76, 61, 62, 59, 55, 104, 94, 85, 59, 71, 63, 65, 66, 113, 144,
            104, 63, 72, 64, 70, 70, 126, 154, 109, 71, 69, 67, 73, 68, 106, 122, 88, 68, 68, 68,
            79, 60, 70, 77, 66, 58, 75, 69, 85, 64, 58, 55, 61, 65, 83, 70, 87, 69, 68, 65, 73, 78,
            90,
        ];

        let mut histogram: HashMap<u8, u32> = HashMap::new();
        for i in 0u8..=255u8 {
            histogram.insert(i, 0);
        }

        for value in buf {
            *histogram
                .get_mut(&value)
                .expect("Unexpected error regarding the histogram Hashmap") += 1;
        }

        let mut output = cumulative_distribution(&histogram);
        let mut expected_output: HashMap<u8, u32> = HashMap::new();
        for i in 0u8..=255u8 {
            expected_output.insert(i, 0);
        }

        expected_output.insert(52, 1);
        expected_output.insert(55, 4);
        expected_output.insert(58, 6);
        expected_output.insert(59, 9);
        expected_output.insert(60, 10);
        expected_output.insert(61, 14);
        expected_output.insert(62, 15);
        expected_output.insert(63, 17);
        expected_output.insert(64, 19);
        expected_output.insert(65, 22);
        expected_output.insert(66, 24);
        expected_output.insert(67, 25);
        expected_output.insert(68, 30);
        expected_output.insert(69, 33);
        expected_output.insert(70, 37);
        expected_output.insert(71, 39);
        expected_output.insert(72, 40);
        expected_output.insert(73, 42);
        expected_output.insert(75, 43);
        expected_output.insert(76, 44);
        expected_output.insert(77, 45);
        expected_output.insert(78, 46);
        expected_output.insert(79, 48);
        expected_output.insert(83, 49);
        expected_output.insert(85, 51);
        expected_output.insert(87, 52);
        expected_output.insert(88, 53);
        expected_output.insert(90, 54);
        expected_output.insert(94, 55);
        expected_output.insert(104, 57);
        expected_output.insert(106, 58);
        expected_output.insert(109, 59);
        expected_output.insert(113, 60);
        expected_output.insert(122, 61);
        expected_output.insert(126, 62);
        expected_output.insert(144, 63);
        expected_output.insert(154, 64);

        for i in 0..=255u8 {
            match expected_output.get(&i) {
                Some(0) => continue,
                Some(v) => assert_eq!(Some(v), output.get(&i)),
                None => assert_eq!(None, output.get(&i)),
            }
        }
    }
}
