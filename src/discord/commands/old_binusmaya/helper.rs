use serenity::builder::CreateSelectMenuOption;

pub async fn academic_period_menu_options(course_menu_list: &serde_json::Value) -> Vec<CreateSelectMenuOption> {
	let mut vec_opt: Vec<CreateSelectMenuOption> = Vec::new();
	let academic_period_list = course_menu_list[0][3].as_array().unwrap();

	academic_period_list.iter().enumerate().for_each(|(i, menu_list)| {
		let mut opt = CreateSelectMenuOption::default();
		opt.label(menu_list[1].as_str().unwrap());
		opt.value(i);

		vec_opt.push(opt);
	});

	vec_opt
}

pub async fn course_menu_options(course_menu_list: &serde_json::Value, academic_period_index: usize) -> Vec<CreateSelectMenuOption> {
	let mut vec_opt: Vec<CreateSelectMenuOption> = Vec::new();
	let course_list = course_menu_list[0][3][academic_period_index].as_array().unwrap();

	course_list.iter().enumerate().filter(|(i, _)| i > &1).for_each(|(i, c)| {
		let mut opt = CreateSelectMenuOption::default();
		opt.label(format!("{} - {}", c["CLASS_SECTION"].as_str().unwrap(), c["COURSE_TITLE_LONG"].as_str().unwrap()));
		opt.value(i);

		vec_opt.push(opt);
	});

	vec_opt
}