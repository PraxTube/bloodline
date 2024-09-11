use std::{
    fs::{read, read_to_string, write},
    path::Path,
};

use petgraph::{
    dot::{Config, Dot},
    prelude::*,
};
use rusqlite::{Connection, Result};

const PATH_TO_DB: &str = "./bloodline.db";

struct Container {
    graph: Graph<String, usize, Directed>,
}

const OUTPUT_FILE: &str = "out.dot";

struct Person {
    id: usize,
    name: String,
    surname: String,
    middlename: Option<String>,
    image: Option<Vec<u8>>,
}

struct Relation {
    person: usize,
    father: usize,
    mother: usize,
}

fn format_pic_str(id: usize) -> String {
    format!("{:03}-pic.jpg", id)
}

fn parse_labels_dot_file(path: &str) {
    fn construct_line(line: &str, parts: Vec<&str>) -> String {
        if let Ok(id) = parts[0].parse::<usize>() {
            if parts[1] != "[" {
                return line.to_string();
            }

            let pic_str = format_pic_str(id);
            if !Path::new(&pic_str).exists() {
                return line.to_string();
            }

            let appendix = format!(" [ image = \"{}\" ]", pic_str);
            line.to_string() + &appendix
        } else {
            line.to_string()
        }
    }

    let content = read_to_string(path).expect("Failed to read file content");
    let mut output_content = String::new();

    for line in content.split("\n") {
        let parts: Vec<&str> = line.trim().split(' ').collect();

        let final_line = construct_line(line, parts);
        output_content += &(final_line + "\n");
    }

    write(path, output_content).expect("Failed to parse dot file");
}

fn construct_graph() -> Result<Graph<String, usize, Directed>> {
    let mut container = Container {
        graph: Graph::default(),
    };

    let conn = Connection::open(PATH_TO_DB)?;

    let mut stmt = conn.prepare("SELECT * FROM person")?;
    let persons = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            surname: row.get(2)?,
            middlename: row.get(3)?,
            image: row.get(4)?,
        })
    })?;

    for person in persons {
        let person = person.unwrap();
        let _index = container
            .graph
            .add_node(format!("{} {}", person.name, person.surname));

        let path = format_pic_str(person.id);
        if let Some(contents) = person.image {
            write(path, contents).expect("Failed to write image to file");
        }
    }

    let mut stmt = conn.prepare("SELECT * FROM relations")?;
    let relations = stmt.query_map([], |row| {
        Ok(Relation {
            person: row.get(0)?,
            father: row.get(1)?,
            mother: row.get(2)?,
        })
    })?;

    for relation in relations {
        let relation = relation.unwrap();
        let father_index = NodeIndex::new(relation.father);
        let mother_index = NodeIndex::new(relation.mother);
        let child_index = NodeIndex::new(relation.person);

        container.graph.add_edge(father_index, child_index, 0);
        container.graph.add_edge(mother_index, child_index, 0);
    }

    Ok(container.graph)
}

fn initialize_database() -> Result<()> {
    let conn = Connection::open(PATH_TO_DB)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS person (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            surname TEXT NOT NULL,
            middlename TEXT,
            image BLOB
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS relations (
            person PERSON PRIMARY KEY,
            father PERSON,
            mother PERSON
        )",
        (),
    )?;

    Ok(())
}

fn dummy_insert() -> Result<()> {
    let conn = Connection::open(PATH_TO_DB)?;

    conn.execute(
        "INSERT INTO person (id, name, surname) VALUES (?1, ?2, ?3)",
        (0, "Herr", "Mustermann"),
    )?;
    conn.execute(
        "INSERT INTO person (id, name, surname) VALUES (?1, ?2, ?3)",
        (1, "Frau", "Mustermann"),
    )?;
    conn.execute(
        "INSERT INTO person (id, name, surname) VALUES (?1, ?2, ?3)",
        (2, "Kind", "Mustermann"),
    )?;

    conn.execute(
        "INSERT INTO relations (person, father, mother) VALUES (?1, ?2, ?3)",
        (2, 0, 1),
    )?;

    // ---

    conn.execute(
        "INSERT INTO person (id, name, surname, image) VALUES (?1, ?2, ?3, ?4)",
        (3, "Me", "Rancic", read("./pic.jpg").unwrap()),
    )?;

    Ok(())
}

fn main() {
    initialize_database().unwrap();
    dummy_insert().unwrap();

    let graph = construct_graph().unwrap();

    write(
        OUTPUT_FILE,
        Dot::with_config(&graph, &[Config::EdgeNoLabel])
            .to_string()
            .as_bytes(),
    )
    .expect("Can't write to output dot file");
    parse_labels_dot_file(OUTPUT_FILE);
}
