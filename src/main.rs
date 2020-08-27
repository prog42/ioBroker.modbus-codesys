#![allow(non_snake_case)]

extern crate xmltree;
extern crate csv;
extern crate regex;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate colored;
    
use colored::*;

use std::fs::File;
use std::io::prelude::*;
use xmltree::Element;
use std::collections::HashMap;

extern crate clap;

// Values can be found in WAGO 750-880 Documentation Chapter 12.2.4 MODBUS-Register-Mapping.
const MARKER_REF_ID: u32 = 0;
const MARKER_REGION_1_START_BYTE_ADR: u32 = 0; // %MW0
const MARKER_REGION_1_MODBUS_START_REG: u32 = 12288;

const HOLDINGS_REF_ID: u32 = 1;
const HOLDINGS_REGION_1_START_BYTE_ADR: u32 = 512; // %IW256
const HOLDINGS_REGION_1_MODBUS_START_REG: u32 = 768;
const HOLDINGS_REGION_2_START_BYTE_ADR: u32 = 1024; // %IW512
const HOLDINGS_REGION_2_MODBUS_START_REG: u32 = 24576;

const INPUTS_REF_ID: u32 = 2;
const INPUTS_REGION_1_START_BYTE_ADR: u32 = 512;   // %QW256
const INPUTS_REGION_1_MODBUS_START_REG: u32 = 256;

#[derive(Debug)]
enum Types {
  Undefined,
  TypeSimple,
  TypeUserdef,
  TypeArray,
  TypeString,
  TypeEnum,
}

impl Default for Types {
  fn default() -> Types { Types::Undefined }
}

#[derive(Debug, Default)]
struct TypeNode {
  type_: Types,
  type_id: u32,
  size: u32,
  name: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CSVEntryHoldings {
  address: u32,
  name: String,
  description: String,
  unit: String,
  #[serde(rename = "type")]
  type_: String,
  #[serde(rename = "len")]
  length: u32,
  factor: u32,
  offset: u32,
  role: String,
  room: String,
  poll: bool,
  #[serde(rename = "wp")]
  writepulse: bool,
}

impl Default for CSVEntryHoldings {
  fn default() -> CSVEntryHoldings {
    CSVEntryHoldings {
      address: 0,
      name: String::from(""),//use clap::App;

      description: String::from(""),
      unit: String::from(""),
      type_: String::from(""),
      length: 1,
      factor: 1,
      offset: 0,
      role: String::from("state"),
      room: String::from(""),
      poll: true,
      writepulse: false
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CSVEntryInputs {
  address: u32,
  name: String,
  description: String,
  unit: String,
  #[serde(rename = "type")]
  type_: String,
  #[serde(rename = "len")]
  length: u32,
  factor: u32,
  offset: u32,
  role: String,
  room: String
}

impl Default for CSVEntryInputs {
  fn default() -> CSVEntryInputs {
    CSVEntryInputs {
      address: 0,
      name: String::from(""),
      description: String::from(""),
      unit: String::from(""),
      type_: String::from(""),
      length: 1,
      factor: 1,
      offset: 0,
      role: String::from("state"),
      room: String::from(""),
    }
  }
}

fn main() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    let cmd_line_matches = clap::App::new("ioBroker-modbus-codesys-convert")
      .version(VERSION)
      
      .arg(clap::Arg::with_name("symbol-xml")
        .short("S")
        .long("symbol-xml")
        .value_name("project.SYM_XML")
        .required(true)
        .takes_value(true))

      .arg(clap::Arg::with_name("symbol-filter")
        .short("F")
        .long("symbol-filter regex")
        .value_name("")
        .required(false)
        .takes_value(true))

      .arg(clap::Arg::with_name("holdings")
        .short("H")
        .long("holdings")
        .value_name("holdings.csv")
        .takes_value(true))
      
      .arg(clap::Arg::with_name("inputs")
        .short("I")
        .long("inputs")
        .value_name("inputs.csv")
        .takes_value(true))
      .get_matches();
    
    // import codesys-2.3 symbol xml file
    let mut types_map = HashMap::new();
    let symbol_xml_filenme = cmd_line_matches.value_of("symbol-xml").unwrap();
    
    let mut symbol_xml_filter_regex_str = ".*";
    if cmd_line_matches.is_present("symbol-filter") {
      symbol_xml_filter_regex_str = cmd_line_matches.value_of("symbol-filter").unwrap();
    }

    let symbol_xml_filter_regex = regex::Regex::new(symbol_xml_filter_regex_str).unwrap();
    
    let symbol_var_list: Element = read_symbol_xml(symbol_xml_filenme, &mut types_map);

    // import holdings csv file
    let holdings_filenme = cmd_line_matches.value_of("holdings").unwrap_or("holdings.csv");
    let csv_holdings = read_csv::<CSVEntryHoldings>(holdings_filenme);
    let mut new_csv_holdings: Vec<CSVEntryHoldings> = Vec::new();

    // import imputs csv file
    let inputs_filenme = cmd_line_matches.value_of("inputs").unwrap_or("inputs.csv");
    let csv_inputs = read_csv::<CSVEntryInputs>(inputs_filenme);
    let mut new_csv_inputs: Vec<CSVEntryInputs> = Vec::new();

    // preapre regex for name mangling
    let replace_dot_regex = regex::Regex::new(r"^\.").unwrap();
    let replace_opening_braces_regex = regex::Regex::new(r"\[|\]").unwrap();
    let replace_closing_braces_regex = regex::Regex::new(r"\]$").unwrap();
    let replace_closing_brace_dot_regex = regex::Regex::new(r"\]\.").unwrap();

    // parse symbol_var_list and generate CSV File
    for child in &symbol_var_list.children {
      match child.attributes.get("RefId") {
        Some(ref ref_id) => {
          let id: u32 = ref_id.parse().unwrap();
          if (id == MARKER_REF_ID) || (id == HOLDINGS_REF_ID) || (id == INPUTS_REF_ID) {
            let mut name = child.text.clone().unwrap();

            if !symbol_xml_filter_regex.is_match(&name) {
              continue;
            }
            // remove dots at beginning
            name = replace_dot_regex.replace_all(&name, "").into_owned();
            let desciption = name.clone();

            // remove braces for name
            name = replace_closing_brace_dot_regex.replace_all(&name, ".").into_owned();;
            name = replace_closing_braces_regex.replace_all(&name, "").into_owned();;
            name = replace_opening_braces_regex.replace_all(&name, ".").into_owned();;

            if (id == MARKER_REF_ID) || (id == HOLDINGS_REF_ID) {
              let mut new_entry = CSVEntryHoldings { ..Default::default() };
              let mut skip_entry = false;
              let mut csv_holdings_iter = csv_holdings.iter();
              match csv_holdings_iter.find(|i| i.name == name) {
                Some(csv_entry_holdings) => {
                  // name found in old csv
                  // use entry
                  new_entry = csv_entry_holdings.clone();
                },
                None => {
                  // name not found
                  // initialize some fields
                  new_entry.description = desciption;
                  new_entry.name = name;
                },
              }
              let offset: u32 = child.attributes.get("Offset").unwrap().parse().unwrap();
              let mut error_string = "";
              match offset_to_address(id, offset) {
                Ok(adr) => new_entry.address = adr,
                Err(why) => { skip_entry = true;
                  error_string = why},
              }

              let type_attr: u32 = child.attributes.get("Type").unwrap().parse().unwrap();
              let length_type = get_modbus_length_type(type_attr, &types_map);
              new_entry.length = length_type.0;
              new_entry.type_ = length_type.1;

              if new_entry.length > 0 {
                if skip_entry {
                  println!("{} {}: {} ({})", "skipping address".red().bold(), offset.to_string().red(), new_entry.name, error_string);
                } else {
                  println!("{} {}: {}", "holdings".green(), new_entry.address.to_string().green().bold(), new_entry.name);
                  new_csv_holdings.push(new_entry);
                }
              }
              
            } else if id == INPUTS_REF_ID {
              let mut new_entry = CSVEntryInputs { ..Default::default() };
              let mut skip_entry = false;
              let mut csv_inputs_iter = csv_inputs.iter();
              match csv_inputs_iter.find(|i| i.name == name) {
                Some(csv_entry_inputs) => {
                  // name found in old csv
                  // use entry
                  new_entry = csv_entry_inputs.clone();
                },
                None => {
                  // name not found
                  // initialize some fields
                  new_entry.description = desciption;
                  new_entry.name = name;
                },
              }
              
              let offset: u32 = child.attributes.get("Offset").unwrap().parse().unwrap();
              let mut error_string = "";
              match offset_to_address(id, offset) {
                Ok(adr) => new_entry.address = adr,
                Err(why) => { skip_entry = true;
                  error_string = why},
              }

              let type_attr: u32 = child.attributes.get("Type").unwrap().parse().unwrap();
              let length_type = get_modbus_length_type(type_attr, &types_map);
              new_entry.length = length_type.0;
              new_entry.type_ = length_type.1;
              
              if new_entry.length > 0 {
                if skip_entry {
                  println!("{} {}: {} ({})", "skipping address".red().bold(), offset.to_string().red(), new_entry.name, error_string);
                } else {
                  println!("{} {}: {}", "inputs".green(), new_entry.address.to_string().green().bold(), new_entry.name);
                  new_csv_inputs.push(new_entry);
                }
              }
            }
          }
        },
        None => {},
      }
    }
    
    let create_output_filename_regex = regex::Regex::new(r"(?P<a>.*)\.(?P<b>.*)$").unwrap();
  
    // write holdings csv file
    let holdings_out_filenme = create_output_filename_regex.replace_all(&holdings_filenme, "$a-out.$b").into_owned();
    write_csv(&holdings_out_filenme, &new_csv_holdings);

    // write inputs csv file
    let inputs_out_filenme   = create_output_filename_regex.replace_all(&inputs_filenme,   "$a-out.$b").into_owned();
    write_csv(&inputs_out_filenme, &new_csv_inputs);
}

fn offset_to_address<'a>(id: u32, offset: u32) -> Result<u32, &'a str> {
  match id {
    MARKER_REF_ID => {
      if (offset % 2) == 0 {
        return Ok(((offset - MARKER_REGION_1_START_BYTE_ADR) / 2) + MARKER_REGION_1_MODBUS_START_REG);
      } else {
        return Err("unaligned")
      }
    }
    HOLDINGS_REF_ID => {
      if (offset % 2) == 0 {
        if (offset >= HOLDINGS_REGION_1_START_BYTE_ADR) && (offset < HOLDINGS_REGION_2_START_BYTE_ADR) {
          return Ok(((offset - HOLDINGS_REGION_1_START_BYTE_ADR) / 2) + HOLDINGS_REGION_1_MODBUS_START_REG);
        } else if offset >= HOLDINGS_REGION_2_START_BYTE_ADR { 
          return Ok(((offset - HOLDINGS_REGION_2_START_BYTE_ADR) / 2) + HOLDINGS_REGION_2_MODBUS_START_REG);
        } else {
          return Err("out of range")
        }
      } else {
        return Err("unaligned");
      }
    }
    INPUTS_REF_ID => {
      if (offset % 2) == 0 {
        if offset >= INPUTS_REGION_1_START_BYTE_ADR {
          return Ok(((offset - INPUTS_REGION_1_START_BYTE_ADR) / 2) + INPUTS_REGION_1_MODBUS_START_REG);
        } else {
          return Err("out of range")
        }
      } else {
        return Err("unaligned");
      }
    }
    _ => {
      return Err("illegal ID")
    }
  }
}

fn get_modbus_length_type(type_attr: u32, types_map: & HashMap<u32, TypeNode>) -> (u32, String) {
  let mut type_ = String::from("");
  let mut length:u32 = 0;
  match types_map.get(&type_attr) {
    Some(typenode) => {
      match typenode.type_ {
        Types::TypeSimple => {
          if typenode.name == "BYTE" {
            type_ = String::from("uint8be");
            length = 1;
          } else if typenode.name == "WORD" {
            type_ = String::from("uint16be");
            length = 1;
          } else if typenode.name == "DWORD" {
            type_ = String::from("uint32sw");
            length = 2;
          } else if typenode.name == "SINT" {
            type_ = String::from("int8be");
            length = 1;
          } else if typenode.name == "USINT" {
            type_ = String::from("uint8be");
            length = 1;
          } else if typenode.name == "INT" {
            type_ = String::from("int16be");
            length = 1;
          } else if typenode.name == "UINT" {
            type_ = String::from("uint16be");
            length = 1;
          } else if typenode.name == "DINT" {
            type_ = String::from("int32sw");
            length = 2;
          } else if typenode.name == "UDINT" {
            type_ = String::from("uint32sw");
            length = 2;
          } else if typenode.name == "REAL" {
            type_ = String::from("floatsw");
            length = 2;
          } else if typenode.name == "TIME" {
            type_ = String::from("uint32sw");
            length = 2;
          } else if typenode.name == "TOD" {
            type_ = String::from("uint32sw");
            length = 2;
          } else if typenode.name == "DATE" {
            type_ = String::from("uint32sw");
            length = 2;
          } else if typenode.name == "DT" {
            type_ = String::from("uint32sw");
            length = 2;
          } else {
            type_ = typenode.name.clone();
          }
        },
        Types::TypeString => {
          type_ = String::from("stringle");
          length = typenode.size;
        },
        _ => {
          type_ = typenode.name.clone();
        }
      }
    },
    _node => {}
  }
  (length, type_)
}

fn read_symbol_xml(filename: &str, types_map: &mut HashMap<u32, TypeNode>) -> Element {
  let mut xml_file = match File::open(filename) {
    Err(why) => panic!("could not open file {}: {}", filename, why),
    Ok(file) => file,
  };

  let mut xml_string = String::new();
  xml_file.read_to_string(&mut xml_string).expect("could not read file");
  let co_de_sys_symbol_table = Element::parse(xml_string.as_bytes()).unwrap();
  
  // parse symbol_type_list and insert in types_map
  let symbol_type_list = co_de_sys_symbol_table.get_child("SymbolTypeList");
  if let Some(ref symbol_type_list_) = symbol_type_list {
    for child in &symbol_type_list_.children {
      let mut node = TypeNode { ..Default::default() };

      if child.name == "TypeSimple" {
        node.type_ = Types::TypeSimple;
        match child.text {
          Some(ref name) => node.name = name.clone(),
          None => {},
        }
      } else if child.name == "TypeUserdef" {
        node.type_ = Types::TypeUserdef;
      } else if child.name == "TypeArray" {
        node.type_= Types::TypeArray;
      } else if child.name == "TypeString" {
        node.type_ = Types::TypeString;
      } else if child.name == "TypeEnum" {
        node.type_ = Types::TypeEnum;
      }

      match child.attributes.get("TypeId") {
        Some(type_id) => node.type_id = type_id.parse().unwrap(),
        None => {},
      }
      match child.attributes.get("Size") {
        Some(size) => node.size = size.parse().unwrap(),
        None => {},
      }        
      types_map.insert(node.type_id, node);
    }
  } 

  return co_de_sys_symbol_table.get_child("SymbolVarList").unwrap().clone();
}

fn read_csv<T>(filename: &str) -> Vec<T>
  where for<'de> T: serde::Deserialize<'de> {
  let csv_string = match File::open(filename) {
    Err(_why) => String::from(""),
    Ok(mut file) => {
      let mut string = String::new();
      file.read_to_string(&mut string).expect("could not read file");
      string
      }
  };

  let mut csv_reader = csv::ReaderBuilder::new()
    .has_headers(true)
    .delimiter(b'\t')
    .flexible(true)
    .from_reader(csv_string.as_bytes());
  
  let mut csv_holdings: Vec<T> = Vec::new();
  
  for result in csv_reader.deserialize() {
    match result {
      Ok(rec) => csv_holdings.push(rec),
      Err(_err) => {},      
    }
  }
  return csv_holdings;
}

fn write_csv<T>(filename: &str, entries: &Vec<T>)
  where T: serde::Serialize {
  let mut writer = csv::WriterBuilder::new()
    .has_headers(true)
    .delimiter(b'\t')
    .flexible(true)
    .from_path(filename).unwrap();
  
  for entry in entries {
    writer.serialize(entry).expect("could not serialize");
  }
  
  writer.flush().expect("could not write file");
}
