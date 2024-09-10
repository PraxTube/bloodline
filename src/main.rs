use std::{fs::File, io::Write};

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

    Ok(())
}

fn main() {
    initialize_database().unwrap();
    dummy_insert().unwrap();

    let graph = construct_graph().unwrap();

    let mut file = match File::create(OUTPUT_FILE) {
        Ok(r) => r,
        Err(err) => panic!("Can't create/open file: '{}', {}", OUTPUT_FILE, err),
    };

    file.write_all(
        Dot::with_config(&graph, &[Config::EdgeNoLabel])
            .to_string()
            .as_bytes(),
    )
    .unwrap_or_else(|_| panic!("Couldn't write to file: '{}'", OUTPUT_FILE));
}
