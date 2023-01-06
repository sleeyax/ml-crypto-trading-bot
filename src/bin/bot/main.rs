use lightgbm::Booster;

fn main() {
    let model_file = "model.trained";
    let booster = Booster::from_file(model_file).unwrap();
    let result = booster
        .predict(vec![
            vec![16502.19], // features (open)
        ])
        .unwrap();
    let score = result[0][0];
    println!("{}", score);
}
