use trees::tr;

fn main(){

	fn parse(v: Vec<&str>) -> trees::Tree<&str> { //uses a list to allow for multi-character functions. Also, I don't want to deal with
													//rust's godawful strings more than I need to
		let operators = vec!["^", "/", "*", "-", "+"]; //in order of operations
		let mut paren_layer = 0u32;
		let mut operator_index: Option<usize> = None; //operator index in v
		let mut operator_priority:i32 = -1;//operator index in operators (-1 if operator index is still none)
		for i in 0..v.len() {
			let s = v[i];
						//paren cases
			if s == "(" {
				paren_layer += 1;
			}
			if s == ")"{
				if paren_layer <= 0 {
					panic!("unbalanced parens") //TODO
				}
				paren_layer -= 1;
			}

			//operator cases
			if paren_layer == 0{
				let priotiy = operators.iter().position(|x| x == &s);
				match priotiy {
					Some (x) => if x as i32 > operator_priority{
						operator_priority = x as i32;
						operator_index = Some(i);
					},
					None => () //should just do nothing
				}
			}
		}
		if paren_layer != 0{
			//panic("unbalanced parens") TODO
		}
		match operator_index {
			Some (x) => return tr(v[x])/parse(v[0..x].to_vec())/parse(v[x+1..].to_vec()),
			None => ()
		}
		if v.len() == 1 {
			return tr(v[0]);
		}
		if v[0] == "(" && v[v.len() - 1] == ")" {
			return parse(v[1..v.len()-1].to_vec());
		}
		return tr(""); //TODO error message
	}
}