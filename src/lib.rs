const MAX_GAUCHOS: usize = 16;

#[derive(Copy, Clone)]
struct Gaucho {
    active: bool,
    x: f64,
    y: f64,
}

struct World {
    gauchos: [Gaucho; MAX_GAUCHOS],
}

static WORLD: World = World {
    gauchos: [Gaucho {
        active: false,
        x: 0.0,
        y: 0.0,
    }; MAX_GAUCHOS],
};

/*
static world: World = World {
    gauchos: [
        Gaucho {
            active: false,
            x: 0.0,
            y: 0.0,
        },
        Gaucho {
            active: false,
            x: 0.0,
            y: 0.0,
        },
        Gaucho {
            active: false,
            x: 0.0,
            y: 0.0,
        },
    ],
};
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
