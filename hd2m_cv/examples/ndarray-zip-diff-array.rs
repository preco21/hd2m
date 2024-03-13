use ndarray::Array3;
use ndarray::Zip;

fn main() {
    type Array3f64 = Array3<f64>;

    const N: usize = 3;
    let a = Array3f64::from_elem((N, N, N), 1.);
    let b = Array3f64::from_elem(a.dim(), 2.);
    let mut c = Array3::<i32>::zeros(a.dim());

    // FINDINGS: Can use different types for the result array
    Zip::from(&mut c).and(&a).and(&b).par_for_each(|c, &a, &b| {
        *c += (a - b) as i32;
    });

    println!("{:?}", c);
}
