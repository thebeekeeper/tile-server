#[macro_use] extern crate nickel;
extern crate rusqlite;
#[macro_use] extern crate log;

use nickel::{Nickel, HttpRouter, StaticFilesHandler};
use nickel::mimes::MediaType;
use rusqlite::Connection;

#[derive(Debug, Default)]
struct Tile {
    pub zoom_level: i32,
    pub tile_column: i32,
    pub tile_row: i32,
    pub data: Vec<u8>,
}

fn main() {
    let mut server = Nickel::new();

    server.get("/tile/:z/:x/:y", middleware! { |request, mut response|
        let z = request.param("z").unwrap();
        let x = request.param("x").unwrap();
        let y = request.param("y").unwrap();
        trace!("tile request: {}, {}, {}", z, x, y);
        let c = Connection::open("sat.mbtiles").unwrap();
        let mut stmt = c.prepare("SELECT * from tiles WHERE zoom_level = ($1) AND tile_column = ($2) AND tile_row = ($3)").unwrap();
                                 
        let tile_iter = stmt.query_map(&[&z, &x, &y], |row| {
            Ok(Tile {
                zoom_level: row.get(0)?,
                tile_column: row.get(1)?,
                tile_row: row.get(2)?,
                data: row.get(3)?,
            })
        }).unwrap();

        // blecch i can't figure out a better way to get the first item out of this iterator
        let mut tile = Tile::default();
        for t in tile_iter {
            tile = t.unwrap();
            break;
        }
        response.set(MediaType::Jpeg);
        tile.data
    });

    server.utilize(StaticFilesHandler::new("public/"));
    server.listen("127.0.0.1:6767").unwrap();
}
