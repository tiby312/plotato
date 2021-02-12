
You can find poloto on [github](https://github.com/tiby312/poloto) and [crates.io](https://crates.io/crates/poloto).


A simple 2D plotting library that outputs graphs to SVG that can be styled using CSS.

Poloto graphs can be stylized using css either directly in the SVG, or from inside of html with an embeded svg. The latter allows the user to dynamically match the svg to their website's theme. The user can take full advantage of CSS, adding highlight on hover, animation, shadows, strokes, etc. 

You can see it in action in this rust book [broccoli-book](https://tiby312.github.io/broccoli_report/)

Here is a simple demo:

<img src="./assets/simple.svg" alt="demo">

```rust
//PIPE me to a file!
fn main() {
    let mut s = poloto::plot(
        "Cows Per Year",
        "Year",
        "Cow",
    );

    let data=[
        [1979.0,10.0],
        [1989.0,12.0],
        [2001.0,13.0],
        [2010.0,4.0]
    ];
    
    s.line("cows", data.iter().map(|x|*x));
    

    s.render(std::io::stdout()).unwrap();
}
```
