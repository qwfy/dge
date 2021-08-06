pub(crate) fn longest_common_prefix(strings: Vec<String>) -> String {
    let mut acc = String::new();
    longest_common_prefix_acc(strings, &mut acc, 0);
    acc
}

fn longest_common_prefix_acc(strings: Vec<String>, acc: &mut String, i: usize) {
    let iths: Vec<_> = strings.iter().map(|s| s.get(i..=i)).collect();
    if iths.len() == 0 {
        ()
    } else {
        match iths[0] {
            None => (),
            Some(c) => {
                let all_equal = iths.iter().all(|x| *x == Some(c));
                if all_equal {
                    acc.push_str(c);
                    longest_common_prefix_acc(strings, acc, i+1)
                } else {
                    ()
                }
            }
        }
    }
}