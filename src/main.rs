use std::ops::Range;

use itertools::Itertools;
use nannou::{glam::Vec3Swizzles, prelude::*};
use rand::Rng;

#[derive(Default, Clone, Copy, Debug)]
struct AABB {
    lb: Vec3,
    ub: Vec3,
}

impl AABB {
    fn draw(&self, draw: &Draw) {
        draw.rect()
            .xy((self.lb + self.ub).xy() / 2.0)
            .wh((self.ub - self.lb).xy())
            .rgba(1.0, 0.0, 0.0, 0.01)
            .stroke(RED)
            .stroke_weight(1.0);
    }

    fn union(&self, other: &AABB) -> AABB {
        AABB {
            lb: self.lb.min(other.lb),
            ub: self.ub.max(other.ub),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Circle {
    translation: Vec3,
    radius: f32,
}

impl Circle {
    fn aabb(&self) -> AABB {
        AABB {
            lb: self.translation - self.radius,
            ub: self.translation + self.radius,
        }
    }

    fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.translation.xy())
            .radius(self.radius)
            .color(LIGHTGREEN);
    }
}

#[derive(Clone, Copy, Debug)]
enum BVHNode {
    Internal {
        // Bounds of this BVH:
        bounds: AABB,
        // Children BVHNodes:
        left: usize,
        right: usize,
    },
    Leaf {
        // Bounds of this BVH:
        bounds: AABB,
        // Contained primitives
        start: usize,
        end: usize, // non inclusive
    },
}

impl BVHNode {
    fn draw(&self, draw: &Draw) {
        let bounds = self.bounds();
        bounds.draw(draw);
    }

    fn bounds(&self) -> AABB {
        match self {
            BVHNode::Internal { bounds, .. } => *bounds,
            BVHNode::Leaf { bounds, .. } => *bounds,
        }
    }
}

#[derive(Debug)]
struct BVH {
    nodes: Vec<BVHNode>,
    circles: Vec<Circle>,
}

impl BVH {
    fn draw(&self, draw: &Draw) {
        for node in &self.nodes {
            node.draw(draw);
        }
        for circle in &self.circles {
            circle.draw(draw);
        }
    }

    fn compute_bounds(&mut self, node_idx: usize) {
        let mut node = self.nodes[node_idx];
        match &mut node {
            BVHNode::Internal {
                bounds,
                left,
                right,
            } => {
                let l = &self.nodes[*left];
                let r = &self.nodes[*right];
                *bounds = l.bounds().union(&r.bounds());
            }
            BVHNode::Leaf { bounds, start, end } => {
                let mut new_bounds = self.circles[*start].aabb();
                for i in *start + 1..*end {
                    new_bounds = new_bounds.union(&self.circles[i].aabb())
                }
                *bounds = new_bounds;
            }
        };
        self.nodes[node_idx] = node;
    }

    fn subdivide(&mut self, node_idx: usize, threshold: usize) {
        let node = self.nodes[node_idx];
        let node = match node {
            BVHNode::Internal {
                bounds,
                left,
                right,
            } => {
                self.subdivide(left, threshold);
                self.subdivide(right, threshold);
                return;
            }
            BVHNode::Leaf { bounds, start, end } => {
                // Don't subdivide if the number of circles within threshold:
                if end - start <= threshold {
                    return;
                }

                // Compute the longest axis, on which we will split
                let extent = bounds.ub - bounds.lb;
                let mut axis = 0;
                if extent.y > extent.x {
                    axis = 1
                };
                if extent.z > extent[axis] {
                    axis = 2
                };

                // Get the median circle
                let split = bounds.lb[axis] + extent[axis] / 2.0;
                let (mut i, mut j) = (start, end - 1);
                while i <= j {
                    if self.circles[i].translation[axis] < split {
                        i += 1;
                    } else {
                        self.circles.swap(i, j);
                        j -= 1;
                    }
                }

                if i == end - 1 || i == start {
                    // Either empty or one sided, so make no changes.
                    // This is probably unreachable given i use the median
                    // and a threshold, but here to be safe.
                    return;
                }

                let left = BVHNode::Leaf {
                    bounds: Default::default(),
                    start: start,
                    end: i,
                };
                let right = BVHNode::Leaf {
                    bounds: Default::default(),
                    start: i,
                    end: end,
                };

                let l = self.nodes.len();
                self.nodes.push(left);
                let r = self.nodes.len();
                self.nodes.push(right);

                self.compute_bounds(l);
                self.compute_bounds(r);
                self.subdivide(l, threshold);
                self.subdivide(r, threshold);

                BVHNode::Internal {
                    bounds: Default::default(),
                    left: l,
                    right: r,
                }
            }
        };

        self.nodes[node_idx] = node;
        self.compute_bounds(node_idx);
    }

    fn new(circles: Vec<Circle>) -> BVH {
        let mut bvh = BVH {
            nodes: vec![BVHNode::Leaf {
                bounds: AABB::default(),
                start: 0,
                end: circles.len(),
            }],
            circles,
        };

        bvh.compute_bounds(0);

        // Subdivide until all BVHs have at most 4 elements
        bvh.subdivide(0, 2);

        bvh
    }
}

#[derive(Debug)]
struct Model {
    _window: window::Id,
    bvh: BVH,
    circles: Vec<Circle>,
}

fn main() {
    nannou::app(model).update(update).run()
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();

    let mut rng = rand::rng();
    let circles = (0..50)
        .map(|_| Circle {
            translation: Vec3::new(
                rng.random_range(-500.0..500.0),
                rng.random_range(-500.0..500.0),
                rng.random_range(-500.0..500.0),
            ),
            radius: rng.random_range(4.0..=5.0),
        })
        .collect_vec();

    let bvh = BVH::new(circles);

    Model {
        _window,
        circles: bvh.circles.clone(),
        bvh,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let win = app.window_rect();
    let draw = app.draw();

    draw.background().color(BLACK);
    model.bvh.draw(&draw);

    let draw = draw.line_mode();
    draw.polyline()
        .points(model.circles.iter().map(|c| c.translation).collect_vec())
        .color(BLUE);

    draw.to_frame(app, &frame).unwrap();
}
