use ndarray::parallel::prelude::*;
use ndarray::Array;
use ndarray::Axis;

fn main() {
    let a = Array::linspace(0., 63., 64).into_shape((4, 16)).unwrap();
    let mut shapes = Vec::new();
    a.axis_chunks_iter(Axis(0), 2)
        .into_par_iter()
        .map(|chunk| {
            println!("Chunk: {:?}", chunk);
            chunk.shape().to_owned()
        })
        .collect_into_vec(&mut shapes);

    println!("All: {:?}", a);
    println!("Shapes: {:?}", shapes);
}
