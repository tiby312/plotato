//!
//! poloto - plot to SVG and style with CSS
//!
//! ### Usage
//!
//! Check out the [github examples](https://github.com/tiby312/poloto/tree/master/examples).
//! * Plots containing NaN or Infinity are ignored.
//! * After 6 plots, the colors cycle back and are repeated.
//!
use core::fmt::Write;

pub use tagger;
mod util;

///The poloto prelude.
pub mod prelude {
    pub use super::wr2;
    pub use super::iter::PlotIterator;
    pub use core::fmt::Write;
    pub use tagger::wr;
}
use core::fmt;
mod render;

use iter::DoubleIterator;

///Contains the [`DoubleIterator`] trait and three different
///implementers of it.
pub mod iter;

///Contains building blocks for create the default svg an styling tags from scratch.
pub mod default_tags {
    use core::fmt;
    pub use super::render::NUM_COLORS;
    pub use super::render::StyleBuilder;

    ///The class of the svg tag.
    pub const CLASS: &str = "poloto";
    ///The width of the svg tag.
    pub const WIDTH: f64 = 800.0;
    ///The height of the svg tag.
    pub const HEIGHT: f64 = 500.0;
    ///The xmlns: `http://www.w3.org/2000/svg`
    pub const XMLNS: &str = "http://www.w3.org/2000/svg";

    ///Returns a function that will write default svg tag attributes.
    pub fn default_svg_attrs<T: fmt::Write>(
    ) -> impl FnOnce(&mut tagger::AttributeWriter<T>) -> Result<(), fmt::Error> {
        use tagger::prelude::*;

        move |w| {
            w.attr("class", CLASS)?
                .attr("width", WIDTH)?
                .attr("height", HEIGHT)?
                .with_attr("viewBox", wr!("0 0 {} {}", WIDTH, HEIGHT))?
                .attr("xmlns", XMLNS)?;
            Ok(())
        }
    }

}



trait PlotTrait {
    fn write_name(&self, a: &mut fmt::Formatter) -> fmt::Result;
    fn iter_first(&mut self) -> &mut dyn Iterator<Item = [f64; 2]>;
    fn iter_second(&mut self) -> &mut dyn Iterator<Item = [f64; 2]>;
}

use fmt::Display;
struct Wrapper2<D: DoubleIterator, F:Display> {
    a: Option<D>,
    b: Option<D::Next>,
    func:F
}

impl<I: DoubleIterator<Item = [f64; 2]>, F: Display> Wrapper2<I, F> {
    fn new(it: I, func: F) -> Self {
        Wrapper2 {
            a: Some(it),
            b: None,
            func
        }
    }
}

impl<D: DoubleIterator<Item = [f64; 2]>, F:Display>
    PlotTrait for Wrapper2<D, F>
{
    fn write_name(&self, a: &mut fmt::Formatter) -> fmt::Result {
        self.func.fmt(a)
    }
    fn iter_first(&mut self) -> &mut dyn Iterator<Item = [f64; 2]> {
        self.a.as_mut().unwrap()
    }

    fn iter_second(&mut self) -> &mut dyn Iterator<Item = [f64; 2]> {
        self.b = Some(self.a.take().unwrap().finish_first());
        self.b.as_mut().unwrap()
    }
}

enum PlotType {
    Scatter,
    Line,
    Histo,
    LineFill,
}

struct Plot<'a> {
    plot_type: PlotType,
    plots: Box<dyn PlotTrait + 'a>,
}

///Keeps track of plots.
///User supplies iterators that will be iterated on when
///render is called.

//Its important to note that most of the time when this library is used,
//every run through the code is first accompanied by one compilation of the code.
//So inefficiencies in dynamically allocating strings using format!() to then
//be just passed to a writer are not that bad seeing as the solution
//would involve passing a lot of closures around.
pub struct Plotter<'a> {
    names:Box<dyn Names+'a>,
    plots: Vec<Plot<'a>>,
    data: Vec<Box<dyn Display + 'a>>,
    css_variables: bool,
    nostyle: bool,
    nosvgtag: bool,
}




/// Convenience macro to reduce code.
/// Shorthand for 'move |w|write!(w,...)`
/// Create a closure that will use write!() with the formatting arguments.
#[macro_export]
macro_rules! wr2 {
    ($($arg:tt)*) => {
        $crate::moveable_format(move |w| write!(w,$($arg)*))
    }
}


//turn this into a macro.
pub fn moveable_format(func: impl Fn(&mut fmt::Formatter) -> fmt::Result) -> impl fmt::Display {
    struct Foo<F>(F);
    impl<F: Fn(&mut fmt::Formatter) -> fmt::Result> fmt::Display for Foo<F> {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            (self.0)(formatter)
        }
    }
    Foo(func)
}

struct NamesStruct<A,B,C>{
    title:A,
    xname:B,
    yname:C
}
impl<A:Display,B:Display,C:Display> Names for NamesStruct<A,B,C>{
    fn write_title(&self,fm:&mut fmt::Formatter)->fmt::Result{
        self.title.fmt(fm)
    }
    fn write_xname(&self,fm:&mut fmt::Formatter)->fmt::Result{
        self.xname.fmt(fm)
    }
    fn write_yname(&self,fm:&mut fmt::Formatter)->fmt::Result{
        self.yname.fmt(fm)
    }
    
    
}

pub trait Names{
    fn write_title(&self,fm:&mut fmt::Formatter)->fmt::Result;
    fn write_xname(&self,fm:&mut fmt::Formatter)->fmt::Result;
    fn write_yname(&self,fm:&mut fmt::Formatter)->fmt::Result;
    
}



///Convenience function for [`Plotter::new()`]
pub fn plot<'a>(title:impl Display+'a,xname:impl Display+'a,yname:impl Display+'a) -> Plotter<'a> {
    Plotter::new(title,xname,yname)
}


impl<'a> Plotter<'a> {
    /// Create a plotter
    ///
    /// # Example
    ///
    /// ```
    /// let mut s=String::new();
    /// let plotter = poloto::Plotter::new(&mut s);
    /// ```
    pub fn new(title:impl Display+'a,xname:impl Display+'a,yname:impl Display+'a) -> Plotter<'a> {
        Plotter {
            names:Box::new(NamesStruct{
                title,
                xname,
                yname
            }),
            plots: Vec::new(),
            css_variables: false,
            nostyle: false,
            nosvgtag: false,
            data: Vec::new(),
        }
    }

    /*
    pub fn with_options(title:impl Display+'a,xname:impl Display+'a,yname:impl Display+'a,p:Options)->Plotter<'a>{

    }
    */

    /* TODO turn these into flags
    /// Create a plotter with no outer svg tag. This is useful
    /// when you want to create your own svg tag with additional attributes.
    /// The default attributes can be retrived from the [`default_tags`] module.
    ///
    /// # Example
    ///
    /// ```
    /// let mut s=String::new();
    /// let plotter = poloto::Plotter::with_no_svg_tag(&mut s);
    /// ```
    pub fn with_no_svg_tag() -> Plotter<'a> {
        let mut s = Plotter::new();
        s.nosvgtag = true;
        s
    }

    /// Create a plotter with no outer svg tag or default style tag.
    /// The default style can be found in the [`default_tags`] module.
    ///
    /// # Example
    ///
    /// ```
    /// let mut s=String::new();
    /// let plotter = poloto::Plotter::with_no_svg_style_tags(&mut s);
    /// ```
    pub fn with_no_svg_style_tags() -> Plotter<'a> {
        let mut s = Plotter::new();
        s.nosvgtag = true;
        s.nostyle = true;
        s
    }
    */

    /// Create a line from plots.
    ///
    /// # Example
    ///
    /// ```
    /// let data=[
    ///         [1.0f64,4.0],
    ///         [2.0,5.0],
    ///         [3.0,6.0]
    /// ];
    /// use poloto::prelude::*;
    /// let mut s=String::new();
    /// let mut plotter = poloto::Plotter::new(&mut s);
    /// plotter.line(|w|write!(w,"cow"),data.iter().map(|&x|x).twice_iter());
    /// ```
    pub fn line(
        &mut self,
        name: impl Display + 'a,
        plots: impl DoubleIterator<Item = [f64; 2]> + 'a,
    ) -> &mut Self {
        self.plots.push(Plot {
            plot_type: PlotType::Line,
            plots: Box::new(Wrapper2::new(plots.into_iter(), name)),
        });
        self
    }

    /// Create a line from plots that will be filled underneath.
    ///
    /// # Example
    ///
    /// ```
    /// let data=[
    ///         [1.0f64,4.0],
    ///         [2.0,5.0],
    ///         [3.0,6.0]
    /// ];
    /// use poloto::prelude::*;
    /// let mut s=String::new();
    /// let mut plotter = poloto::Plotter::new(&mut s);
    /// plotter.line_fill(|w|write!(w,"cow"),data.iter().map(|&x|x).twice_iter());
    /// ```
    pub fn line_fill(
        &mut self,
        name:impl Display + 'a,
        plots: impl DoubleIterator<Item = [f64; 2]> + 'a,
    ) -> &mut Self {
        self.plots.push(Plot {
            plot_type: PlotType::LineFill,
            plots: Box::new(Wrapper2::new(plots.into_iter(), name)),
        });
        self
    }

    /// Create a scatter plot from plots.
    ///
    /// # Example
    ///
    /// ```
    /// let data=[
    ///         [1.0f64,4.0],
    ///         [2.0,5.0],
    ///         [3.0,6.0]
    /// ];
    /// use poloto::prelude::*;
    /// let mut s=String::new();
    /// let mut plotter = poloto::Plotter::new(&mut s);
    /// plotter.scatter(|w|write!(w,"cow"),data.iter().map(|&x|x).twice_iter());
    /// ```
    pub fn scatter(
        &mut self,
        name: impl Display + 'a,
        plots: impl DoubleIterator<Item = [f64; 2]> + 'a,
    ) -> &mut Self {
        self.plots.push(Plot {
            plot_type: PlotType::Scatter,
            plots: Box::new(Wrapper2::new(plots.into_iter(), name)),
        });
        self
    }

    /// Create a histogram from plots.
    /// Each bar's left side will line up with a point
    ///
    /// # Example
    ///
    /// ```
    /// let data=[
    ///         [1.0f64,4.0],
    ///         [2.0,5.0],
    ///         [3.0,6.0]
    /// ];
    /// use poloto::prelude::*;
    /// let mut s=String::new();
    /// let mut plotter = poloto::Plotter::new(&mut s);
    /// plotter.histogram(|w|write!(w,"cow"),data.iter().map(|&x|x).twice_iter());
    /// ```
    pub fn histogram(
        &mut self,
        name: impl Display + 'a,
        plots: impl DoubleIterator<Item = [f64; 2]> + 'a,
    ) -> &mut Self {
        self.plots.push(Plot {
            plot_type: PlotType::Histo,
            plots: Box::new(Wrapper2::new(plots.into_iter(), name)),
        });
        self
    }
    

    /// User can inject some svg elements using this function.
    /// They will be inserted right after the svg and default svg tags.
    ///
    /// You can override the css in regular html if you embed the generated svg.
    /// This gives you a lot of flexibility giving your the power to dynamically
    /// change the theme of your svg.
    ///
    /// However, if you want to embed the svg as an image, you lose this ability.
    /// If embedding as IMG is desired, instead the user can insert a custom style into the generated svg itself.
    ///
    pub fn with_text(&mut self, inner:impl Display + 'a) -> &mut Self {
        self.data.push(Box::new(
            inner,
        ));
        self
    }

    /// Instead of the default style, use one that adds variables.
    ///
    /// This injects [`default_tags::default_styling_variables`] instead of
    /// the default [`default_tags::default_styling`].
    ///
    /// If you embed the generated svg into a html file,
    /// then you can add this example:
    /// ```css
    /// .poloto{
    ///    --poloto_bg_color:"black";
    ///    --poloto_fg_color:"white;
    ///    --poloto_color0:"red";
    ///    --poloto_color1:"green";
    ///    --poloto_color2:"yellow";
    ///    --poloto_color3:"orange";
    ///    --poloto_color4:"purple";
    ///    --poloto_color5:"pink";
    ///    --poloto_color6:"aqua";
    ///    --poloto_color7:"red";
    /// }
    /// ```  
    /// By default these variables are not defined, so the svg falls back on some default colors.
    pub fn with_css_variables(&mut self) -> &mut Self {
        self.css_variables = true;
        self
    }

    pub fn render_to_string(self)->Result<String,fmt::Error>{
        let mut s=String::new();
        self.render(&mut s)?;
        Ok(s)
    }
    pub fn render_fmt(self,f:&mut fmt::Formatter)->fmt::Result{
        self.render(f)?;
        Ok(())
    }
    pub fn render_io<T:std::io::Write>(self,writer:T)->Result<T,fmt::Error>{
        self.render(tagger::upgrade(writer)).map(|x|x.inner)
    }
    /// Render the svg to the writer.
    ///
    /// Up until now, nothing has been written to the writer. We
    /// have just accumulated a list of commands and closures. This call will
    /// actually call all the closures and consume all the plot iterators.
    pub fn render<T:fmt::Write>(self, writer:T) -> Result<T, fmt::Error>
    {
        let Plotter {
            names,
            plots,
            css_variables,
            nostyle,
            nosvgtag,
            data,
        } = self;
        let mut root = tagger::Element::new(writer);

        use default_tags::*;

        if nosvgtag {
            if !nostyle {
                if css_variables {
                    write!(root,"{}",StyleBuilder::new().build_with_css_variables())?;
                } else {
                    write!(root,"{}",StyleBuilder::new().build())?;
                }
            }

            render::render(root.get_writer(), data, plots, names)?;
        } else {
            root.elem("svg", |writer| {
                let svg = writer.write(|w| {
                    default_svg_attrs()(w)?;

                    Ok(w)
                })?;
                if !nostyle {
                    if css_variables {
                        write!(svg,"{}",StyleBuilder::new().build_with_css_variables())?;
                    } else {
                        write!(svg,"{}",StyleBuilder::new().build())?;
                    }
                }

                render::render(svg.get_writer(), data, plots, names)?;
                Ok(svg)
            })?;
        }
        Ok(root.into_writer())
    }
}
