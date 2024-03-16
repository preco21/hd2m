use ndarray::Array2;
use ndarray::Zip;

fn main() {
    type Array2f64 = Array2<f64>;

    const N: usize = 2000;
    let a = Array2f64::from_elem((N, N), 1.);
    let b = Array2f64::from_elem(a.dim(), 2.);
    let mut c = Array2::<i32>::zeros(a.dim());

    let start = std::time::Instant::now();
    // FINDINGS: Can use different types for the result array
    Zip::from(&mut c).and(&a).and(&b).par_for_each(|c, &a, &b| {
        *c += (a - b) as i32;
    });
    println!("Elapsed (parallel): {:?}", start.elapsed());

    let start = std::time::Instant::now();
    Zip::from(&mut c).and(&a).and(&b).for_each(|c, &a, &b| {
        *c += (a - b) as i32;
    });
    println!("Elapsed (normal): {:?}", start.elapsed());

    println!("{:?}", c);
}
