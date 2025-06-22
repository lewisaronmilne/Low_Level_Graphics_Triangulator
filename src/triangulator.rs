use crate::{XY};

pub fn calc(points: &[XY]) -> Vec<(XY, XY, XY)>
{
    let rankygon = Rankygon::new(points);
    let monos = RankygonToMonos::convert(rankygon);
    let mut triangles = Vec::<(XY, XY, XY)>::new();
    for mono in monos
        { triangles.append(&mut MonotoneToTriangles::convert(mono)); }
    return triangles;
}

#[derive(Debug, Clone)]
struct RankyPoint<'a>
{
    xy: XY,
    path: &'a Rankygon,
    index: usize,
}

impl<'a> RankyPoint<'a>
{
    fn before(&self) -> RankyPoint<'a>
        { return self.path.get_adjacent(self.index, -1) }

    fn after(&self) -> RankyPoint<'a>
        { return self.path.get_adjacent(self.index, 1) }
}

#[derive(Debug, Clone)]
struct Rankygon
{
    points: Vec<XY>,
    rank_to_index: Vec<usize>,
}

impl Rankygon
{
    fn new(points: &[XY]) -> Rankygon
    {
        let path_length = points.len();

        let mut ranky = Rankygon
        {
            points: points.to_vec(),
            rank_to_index: Vec::with_capacity(path_length),
        };

        for i in 0..path_length
            { ranky.rank_to_index.push(i) }; // [0, 1, 2, 3, ..., path_length - 1]
            
        ranky.rank_to_index.sort_by(|a, b|
        { 
            use std::cmp::Ordering;
            let a = &points[*a]; let b = &points[*b];
            if a.x < b.x { return Ordering::Less; }
            else if a.x > b.x { return Ordering::Greater; }
            else if a.y < b.y { return Ordering::Less; }
            else if a.y > b.y { return Ordering::Greater; }
            else { return Ordering::Equal; }
        }); 

        return ranky;
    }

    fn index(&self, index: usize) -> RankyPoint
    {
        RankyPoint
        {
            xy: self.points[index],
            path: &self,
            index: index,
        }
    }

    fn rank(&self, rank: usize) -> RankyPoint
    {
        self.index(self.rank_to_index[rank])
    }

    fn get_adjacent(&self, index: usize, amount: i32) -> RankyPoint
    {
        self.index(((index as i32) + amount).rem_euclid(self.points.len() as i32) as usize)
    }

    fn len(&self) -> usize
    {
        self.points.len()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ChainOption { Top, Bottom, Both }

#[derive(Debug, Clone)] 
struct MonoPoint
{
    xy: XY,
	chain: ChainOption, 
}

#[derive(Debug, Clone)] 
struct Monotone
{
    points: Vec<MonoPoint>
}

impl Monotone
{
    fn new() -> Monotone
    {
        Monotone { points: Vec::new() }
    }

    fn push(&mut self, xy: XY, chain: ChainOption)
    {
        self.points.push(MonoPoint{ xy, chain });
    }
}

use std::collections::HashMap;

#[derive(Debug, Clone)] 
struct RankygonToMonos
{
    rankygon: Rankygon,
    monos: Vec<Monotone>,
    chain_followers: HashMap::<usize /*Follower Index*/, (usize /*Mono Index*/, ChainOption)>,
    chain_mergers: HashMap::<usize /*Follower Index*/, (usize /*Top Mono Index*/, usize /*Bottom Mono Index*/)>,
    merged_monos: HashMap::<usize /*Mono Index*/, (usize /* Other Mono Index*/, bool /* is lookup mono the top mono */)>,
    mono_quads: HashMap::<usize /*Mono Index*/, (usize /*TopFront*/, usize /*TopBack*/, usize /*BottomFront*/, usize /*BottomBack*/)>
}

impl RankygonToMonos
{
    fn convert(rankygon: Rankygon) -> Vec<Monotone>
    {
        let mut container = RankygonToMonos
        {
            rankygon: rankygon,
            monos: Vec::<Monotone>::new(),
            chain_followers: HashMap::<usize, (usize, ChainOption)>::new(),
            chain_mergers: HashMap::<usize, (usize, usize)>::new(),
            merged_monos: HashMap::<usize, (usize, bool)>::new(),
            mono_quads: HashMap::<usize, (usize, usize, usize, usize)>::new()
        };

        for r in 0..container.rankygon.len()
		{
            let current_index = container.rankygon.rank(r).index;

            if container.chain_followers.contains_key(&current_index)
                { container.edges_across(r); }
            else if container.chain_mergers.contains_key(&current_index)
                { container.edges_backward(r);}
            else 
                { container.edges_forward(r); }
		}

        return container.monos;
    }

    fn edges_forward(&mut self, current_rank: usize)
    {
        let (current_index, current_xy, top_follower_index, bottom_follower_index) =
        {
            let current = self.rankygon.rank(current_rank);
            let a = current.before(); let b = current.after();
            if a.xy.y < b.xy.y 
                { (current.index, current.xy, a.index, b.index) } 
            else 
                { (current.index, current.xy, b.index, a.index) }
        };

        let point_within_triangle = |p: &XY, a: &XY, b: &XY, c: &XY| -> bool
        { 
            let w1 = ( a.x*(c.y-a.y) + (p.y-a.y)*(c.x-a.x) - p.x*(c.y-a.y) ) / ( (b.y-a.y)*(c.x-a.x) - (b.x-a.x)*(c.y-a.y) );
            let w2 = ( p.y - a.y - w1*(b.y-a.y) ) / (c.y-a.y);
            return (w1 >= 0.0) && (w2 >= 0.0) && (w1 + w2 <= 1.0);
        };
        		
		let mut split_mono_index = None;
		for (m, q) in self.mono_quads.iter()
		{
            let tf = &self.rankygon.index(q.0).xy;
            let tb = &self.rankygon.index(q.1).xy;
            let bf = &self.rankygon.index(q.2).xy;
            let bb = &self.rankygon.index(q.3).xy;

			if point_within_triangle(&current_xy, tf, tb, bb) || point_within_triangle(&current_xy, tf, bb, bf)
			{ 
                split_mono_index = Some(*m);
                break;
            }
		}

        if split_mono_index == None
        {
            let mut mono = Monotone::new();
            mono.push(current_xy, ChainOption::Both);
            self.monos.push(mono);
            let mono_index = self.monos.len() - 1;

            self.mono_quads.insert(mono_index, (0, 0, 0, 0));
            let current_index = current_index;
            self.setup_followers(current_index, top_follower_index, mono_index, ChainOption::Top);
            self.setup_followers(current_index, bottom_follower_index, mono_index, ChainOption::Bottom);
        }
        else
        {
            let split_mono_index = split_mono_index.unwrap();

            if !self.merged_monos.contains_key(&split_mono_index)
            {
                let (split_mono_bottom_prev_added, split_mono_bottom_follower) = 
				{
					let q = self.mono_quads.get(&split_mono_index).unwrap();
					(q.2, q.3)
				};
                
                self.chain_followers.remove(&split_mono_bottom_follower);
                self.monos[split_mono_index].push(current_xy, ChainOption::Bottom);
                self.setup_followers(current_index, top_follower_index, split_mono_index, ChainOption::Bottom);

                let mut mono = Monotone::new();
                mono.push(self.rankygon.index(split_mono_bottom_prev_added).xy, ChainOption::Both);
                mono.push(current_xy, ChainOption::Top);
                self.monos.push(mono);
                let mono_index = self.monos.len() - 1;

                self.mono_quads.insert(mono_index, (0, 0, 0, 0));
                self.setup_followers(split_mono_bottom_prev_added, bottom_follower_index, mono_index, ChainOption::Top);
                self.setup_followers(split_mono_bottom_prev_added, split_mono_bottom_follower, mono_index, ChainOption::Bottom);
            }
            else
            {
                let (other_mono_index, split_mono_top_or_bottom) = self.merged_monos.remove(&split_mono_index).unwrap();
                self.merged_monos.remove(&other_mono_index);

                let (top_mono_index, bottom_mono_index) = 
					if split_mono_top_or_bottom
						{(split_mono_index, other_mono_index)}
					else 
						{(other_mono_index, split_mono_index)};

                self.monos[top_mono_index].push(current_xy, ChainOption::Bottom);
                self.monos[bottom_mono_index].push(current_xy, ChainOption::Top);
                
                self.setup_followers(current_index, top_follower_index, top_mono_index, ChainOption::Bottom);
                self.setup_followers(current_index, bottom_follower_index, bottom_mono_index, ChainOption::Top);
            }
        }
    }

    fn edges_across(&mut self, current_rank: usize)
    {
        let current = self.rankygon.rank(current_rank);
        let (mut mono_index, chain_type) = self.chain_followers.remove(&current.index).unwrap();

        if self.merged_monos.contains_key(&mono_index)
        {
            let (other_mono_index, _) = self.merged_monos.remove(&mono_index).unwrap();
            self.merged_monos.remove(&other_mono_index);
            self.monos[mono_index].push(current.xy, ChainOption::Both);
            mono_index = other_mono_index
        };

        let mono = &mut self.monos[mono_index];
        mono.push(current.xy, chain_type);

        let follower_index = 
        {
            let a = current.before(); let b = current.after();
            if a.xy.x < b.xy.x { b.index }
            else if a.xy.x > b.xy.x { a.index }
            else if a.xy.y < b.xy.y { b.index }
            else { a.index  }
        };

        let current_index = current.index;
        self.setup_followers(current_index, follower_index, mono_index, chain_type);
    }

    fn edges_backward(&mut self, current_rank: usize)
    {
        let current = self.rankygon.rank(current_rank);
        let (top_mono_index, bottom_mono_index) = self.chain_mergers.remove(&current.index).unwrap();

        if top_mono_index == bottom_mono_index
		{ 
            self.monos[top_mono_index].push(current.xy, ChainOption::Both);
        }
        else if !self.merged_monos.contains_key(&top_mono_index)
        {
            self.merged_monos.insert(top_mono_index, (bottom_mono_index, true));
            self.merged_monos.insert(bottom_mono_index, (top_mono_index, false));

            {
                let top_quad_clone = self.mono_quads.get(&top_mono_index).unwrap().clone();
                let bottom_quad = self.mono_quads.get_mut(&bottom_mono_index).unwrap();
                bottom_quad.0 = top_quad_clone.0;
                bottom_quad.1 = top_quad_clone.1;
            }

            {
                let bottom_quad_clone = self.mono_quads.get(&bottom_mono_index).unwrap().clone();
                let top_quad = self.mono_quads.get_mut(&top_mono_index).unwrap();
                top_quad.2 = bottom_quad_clone.2;
                top_quad.3 = bottom_quad_clone.3;
            }

            self.monos[top_mono_index].push(current.xy, ChainOption::Bottom);
			self.monos[bottom_mono_index].push(current.xy, ChainOption::Top);
        }
        else
        {
            self.merged_monos.remove(&top_mono_index);
            self.merged_monos.remove(&bottom_mono_index);
            
            self.monos[top_mono_index].push(current.xy, ChainOption::Both);
            self.monos[bottom_mono_index].push(current.xy, ChainOption::Both);
        }
    }

    fn setup_followers(&mut self, current_index: usize, follower_index: usize, mono_index: usize, chain_type: ChainOption)
    {
        let quad = self.mono_quads.get_mut(&mono_index).unwrap();
		if chain_type == ChainOption::Top
			{ quad.0 = current_index; quad.1 = follower_index; }
		else 
			{ quad.2 = current_index; quad.3 = follower_index; }

        if !self.chain_followers.contains_key(&follower_index)
		{ 
            self.chain_followers.insert(follower_index, (mono_index, chain_type));
        }
        else
        { 
            let (already_mono_index, _) = self.chain_followers.remove(&follower_index).unwrap();
            if chain_type == ChainOption::Top
                { self.chain_mergers.insert(follower_index, (already_mono_index, mono_index)); }
            else
                { self.chain_mergers.insert(follower_index, (mono_index, already_mono_index)); }
        }
    }
}

struct MonotoneToTriangles;

impl MonotoneToTriangles
{
    fn convert(mono: Monotone) -> Vec<(XY, XY, XY)> 
    {
        let points = &mono.points;
        let mut triangles = Vec::<(XY, XY, XY)>::new();
        let mut a = &points[0];
        let mut b = &points[1];

        let mut skipped_num = 0;
        for i in 2..points.len() 
        {
            let c = &points[i];

            let side_of_line = (c.xy.x - a.xy.x) * (b.xy.y - a.xy.y) - (c.xy.y - a.xy.y) * (b.xy.x - a.xy.x);
            if (b.chain == ChainOption::Top) && (side_of_line <= 0.0) || (b.chain == ChainOption::Bottom) && (side_of_line >= 0.0)
            {
                if b.chain == ChainOption::Top 
                    { triangles.push((a.xy, b.xy, c.xy)); }
                else 
                    { triangles.push((a.xy, c.xy, b.xy)); };

                if skipped_num == 0
                {
                    if b.chain == c.chain 
                        { b = c; }
                    else    
                        { a = b; b = c; };
                }
                else 
                {
                    MonotoneToTriangles::skipped_section(points, &mut triangles, i-skipped_num-1, i);
                    skipped_num = 0;

                    if b.chain == c.chain 
                        { b = c; }
                    else
                        { a = &mono.points[i-1]; b = c; }
                }
            } 
            else 
                { skipped_num += 1; }
        }

        triangles
    }

    fn skipped_section(points: &Vec<MonoPoint>, triangles: &mut Vec<(XY, XY, XY)>, front_index: usize, back_index: usize)
    { 
        if front_index + 1 == back_index
            { return; }

        let chain = points[front_index].chain;
        let front_point = points[front_index].xy;
        let back_point = points[back_index].xy;

        let mut least_ear_index = front_index + 1;
        let mut least_ear_point = points[least_ear_index].xy;
        for i in front_index+2..back_index
        {
            let check_ear_point = points[i].xy;
            if (chain == ChainOption::Top) && (check_ear_point.y > least_ear_point.y) || 
               (chain == ChainOption::Bottom) && (check_ear_point.y < least_ear_point.y)
            {
                least_ear_index = i;
                least_ear_point = check_ear_point;
            };
        };

        if chain == ChainOption::Top
            { triangles.push((front_point, least_ear_point, back_point)) }
        else
            { triangles.push((front_point, back_point, least_ear_point)) }
        
        MonotoneToTriangles::skipped_section(points, triangles, front_index, least_ear_index);
        MonotoneToTriangles::skipped_section(points, triangles, least_ear_index, back_index);
    }
}