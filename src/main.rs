use clap::{Arg, App};

use std::path::Path;
use std::fs::File;

use std::io::Read;
use std::io::BufReader;
use byteorder::{ReadBytesExt, LittleEndian};
use std::cell::RefCell;

use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

use svg::Document as SVGDocument;
use svg::node::element::Group as SVGGroup;
use svg::node::element::Path as SVGPath;
use svg::node::element::path::Data as SVGData;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
enum LogLevel {
  SILENT,
  ERROR,
  WARN,
  INFO,
  DEBUG,
  TRACE
}

impl std::fmt::Display for LogLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

const X_MAX: u32 = 1404;
const Y_MAX: u32  = 1872;

#[derive(Debug)]
struct Point {
  x: f32,
  y: f32,
  speed: f32,
  direction: f32,
  width: f32,
  pressure: f32
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(i32)]
enum BrushType {
  // supports pressure
  Paintbrush = 0,
  // supports pressure, tilt
  PencilTilt = 1,
  // supports pressure
  Pen = 2,
  // supports pressure, tilt
  Marker = 3,
  // no supported modifiers
  Fineliner = 4,
  // no supported modifiers, always black
  Highlighter = 5,
  // no supported modifiers
  Eraser = 6,
  // no supported modifiers
  PencilSharp = 7,
  // no supported modifiers
  RubberArea = 8,
  // no supported modifiers
  EraseAll = 9,
  // no supported modifiers
  SelectionBrush1 = 10,
  // no supported modifiers
  SelectionBrush2 = 11,
  Paintbrush2 = 12,
  MechanicalPencil = 13,
  Pencil2 = 14,
  BallpointPen2 = 15,
  Marker2 = 16,
  Fineliner2 = 17,
  Highlighter2 = 18,
  Calligraphy = 21

}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(i32)]
enum BrushColor {
  Black = 0,
  Grey = 1,
  White = 2
}

#[derive(Debug)]
struct Line {
  brush_type: BrushType,
  brush_color: BrushColor,
  padding: u32,
  unknown: f32,
  brush_size: f32,
  num_points: i32,
  points: Vec<Point>
}

#[derive(Debug)]
struct Layer {
  num_lines: i32,
  lines: Vec<Line>
}

fn main() -> Result<(), String> {
  let opts = App::new("rm2svg")
    .version("0.1.0")
    .author("Dan Shick <dan.shick@gmail.com>")
    .about("Covert .rm v5 files to SVG")
    .arg(Arg::new("input")
      .short('i')
      .long("input")
      .value_name("INPUT")
      .about("Specifies an .rm v5 input file")
      .required(true)
      .takes_value(true))
    .arg(Arg::new("output")
      .short('o')
      .long("output")
      .value_name("OUTPUT")
      .about("Specifies an SVG output file")
      .required(false)
      .takes_value(true))
    .arg(Arg::new("verbose")
      .short('v')
      .multiple_occurrences(true)
      .about("Sets the level of verbosity"))
    .get_matches();

  let logger = get_logger(opts.occurrences_of("verbose"));
  logger(LogLevel::INFO, format!("logger initialized"));

  return opts.value_of("input")
    .map_or_else(|| Err(String::from("no input provided")), |i| get_input_file(i, &logger))
    .and_then(|file| {
      logger(LogLevel::INFO, format!("got file"));
      return parse_file(file, &logger);
    })
    .and_then(|layers| {
      return render_svg(layers);
    })
    .and_then(|svg| {
      svg::save("image.svg", &svg).unwrap();
      return Ok(());
    });

}

fn get_input_file(file: &str, logger: &dyn Fn(LogLevel, String)) -> Result<File, String> {
  logger(LogLevel::INFO, format!("Input is {}", file));
  let file_path = Path::new(file);
  if !file_path.is_file() {
    return Err(String::from("input file does not exist"));
  }
  match File::open(&file_path) {
    Err(why) => Err(format!("couldn't open {}, {}", file_path.display(), why)),
    Ok(file) => Ok(file),
  }
}

const HEADER: &str = "reMarkable .lines file, version=5          ";

fn parse_file(file: File, logger: &dyn Fn(LogLevel, String)) -> Result<Vec<Layer>, String>{

  logger(LogLevel::INFO, format!("parsing file"));

  let mut buffer = Vec::new();
  let reader = RefCell::new(BufReader::new(file));
  reader.borrow_mut().by_ref().take(HEADER.len() as u64).read_to_end(&mut buffer).unwrap();
  logger(LogLevel::DEBUG, format!("actual header is: {:?}", &*buffer));
  logger(LogLevel::DEBUG, format!("expected header is: {:?}", HEADER.as_bytes()));

  if &*buffer != HEADER.as_bytes() {
    return Err(format!("header does not match .rm v5 file"));
  }

  let num_layers: i32 = reader.borrow_mut().read_i32::<LittleEndian>().unwrap();

  let layers = std::iter::repeat_with(||{
    Layer {
      num_lines: reader.borrow_mut().read_i32::<LittleEndian>().unwrap(),
      lines: Vec::<Line>::new()
    }
  })
  .take(num_layers as usize)
  .map(|layer| {
      Layer {
        num_lines: layer.num_lines,
        lines: std::iter::repeat_with(||{
          let mut b_reader = reader.borrow_mut();
          return Line {
            brush_type: BrushType::try_from(b_reader.read_i32::<LittleEndian>().unwrap()).unwrap(),
            brush_color: BrushColor::try_from(b_reader.read_i32::<LittleEndian>().unwrap()).unwrap(),
            padding: b_reader.read_u32::<LittleEndian>().unwrap(),
            unknown: b_reader.read_f32::<LittleEndian>().unwrap(),
            brush_size: b_reader.read_f32::<LittleEndian>().unwrap(),
            num_points: b_reader.read_i32::<LittleEndian>().unwrap(),
            points: Vec::<Point>::new()
          };
        })
        .take(layer.num_lines as usize)
        .map(|line| {
          return Line {
            points: std::iter::repeat_with(||{
              let mut b_reader = reader.borrow_mut();
              return Point{
                x: b_reader.read_f32::<LittleEndian>().unwrap(),
                y: b_reader.read_f32::<LittleEndian>().unwrap(),
                speed: b_reader.read_f32::<LittleEndian>().unwrap(),
                direction: b_reader.read_f32::<LittleEndian>().unwrap(),
                width: b_reader.read_f32::<LittleEndian>().unwrap(),
                pressure: b_reader.read_f32::<LittleEndian>().unwrap()
              }
            })
            .take(line.num_points as usize)
            .collect::<Vec<_>>(),
            ..line
          }
        })
        .collect::<Vec<_>>()
      }
  })
  .collect::<Vec<_>>();

  logger(LogLevel::TRACE, format!("{:#?}", layers));

  return Ok(layers);
}

fn render_svg(layers: Vec<Layer>) -> Result<SVGDocument, String>{
  return Ok(layers.iter().fold(
    SVGDocument::new()
      .set("width", X_MAX)
      .set("height", Y_MAX)
      .set("viewBox", (0, 0, X_MAX, Y_MAX)),
    |acc_svg, next_layer| {
      return acc_svg.add(next_layer.lines.iter().fold(
        SVGGroup::new(),
        |acc_group, next_line| {
          return acc_group.add(
            SVGPath::new()
              .set("fill", "none")
              .set("stroke", "black")
              .set("stroke-width", 3)
              .set("stroke-linejoin", "round")
              .set("stroke-linecap", "round")
              .set("d", next_line.points.iter().enumerate().fold(
                SVGData::new(),
              |acc_data, (index, next_point)| {
                if index == 0 { return acc_data.move_to((next_point.x, next_point.y)); }
                return acc_data.line_to((next_point.x, next_point.y));
              })
            )
          );
        })
      );
    })
  );
}

fn get_logger(verbosity: u64) -> impl Fn(LogLevel, String) -> () {
  move |level, message| {
    if verbosity < level as u64 { return (); }
    println!("[{}]: {}", level.to_string(), message);
  }
}
