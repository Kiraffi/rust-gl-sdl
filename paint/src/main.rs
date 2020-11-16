extern crate sdl2;
extern crate gl;

extern crate render_gl;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use std::ffi::CString;
use std::ffi::CStr;

pub struct ShaderData
{
	_pos_x: f32,
	_pos_y: f32,
	_col: u32,
	_size: f32
}


fn clamp(value: f32, min: f32, max: f32) ->f32
{
	let mut v = if value > min { value } else { min };
	v = if v < max { v } else { max };
	return v;
}

fn get_u32_agbr_color(r: f32, g: f32, b: f32, a: f32) -> u32
{
	let r = clamp(r, 0.0f32, 1.0f32);
	let g = clamp(g, 0.0f32, 1.0f32);
	let b = clamp(b, 0.0f32, 1.0f32);
	let a = clamp(a, 0.0f32, 1.0f32);

	let mut v = 0u32;
	v += (r * 255.0f32) as u32;
	v += ((g * 255.0f32) as u32) << 8u32;
	v += ((b * 255.0f32) as u32) << 16u32;
	v += ((a * 255.0f32) as u32) << 24u32;

	return v;
}



struct App 
{
	
	window_width: i32,
	window_height: i32,
	vsync: bool,

	_sdl: sdl2::Sdl,
	video: sdl2::VideoSubsystem,
	sdl_timer: sdl2::TimerSubsystem,
	window: sdl2::video::Window,
	event_pump: sdl2::EventPump,

	//gl: *const std::os::raw::c_void,
	_gl_context: sdl2::video::GLContext
}

impl App
{
	pub fn init(window_width: i32, window_height: i32, window_name: &str, vsync: bool) -> Result<Self, String>
	{
		/*
		if width == 1
		{
			return Err("failed to initialize".to_string());
		}
*/
		let sdl: sdl2::Sdl  = sdl2::init().unwrap();
		let video: sdl2::VideoSubsystem = sdl.video().unwrap();
		let sdl_timer: sdl2::TimerSubsystem = sdl.timer().unwrap();
		let window;
		match video.window(window_name, window_width as u32, window_height as u32)
		.resizable()
		.opengl()
		.build()
		{
			Ok(v) => 
			{
				window = v; 
			} 
			Err(e) => 
			{ 
				println!("Error: {}", e); 
				return Err("Failed to build window!".to_string()); 
			}
		}
	
		let gl_attr = video.gl_attr();
	
		gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
		gl_attr.set_context_version(4, 5);
	
		gl_attr.set_context_flags().debug().set();
		
	
		let _gl_context = window.gl_create_context()?;
		let _gl = gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);
	
		
	
		let version;
		match unsafe
		{
			let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _)
				.to_bytes()
				.to_vec();
			String::from_utf8(data)
		}
		{
			Ok(v) => 
			{
				version = v; 
			} 
			Err(e) => 
			{ 
				println!("Error: {}", e); 
				return Err("Failed to read version data from gl!".to_string()); 
			}
		}
	
		println!("OpenGL version {}", version);
	
	
		unsafe
		{
			gl::Viewport(0, 0, window_width, window_height);
			gl::ClearColor(0.2, 0.3, 0.5, 1.0);
			gl::ClearDepth(1.0);
			// Swapping up and down just messes things up like in renderdoc....
			//gl::ClipControl(gl::UPPER_LEFT, gl::ZERO_TO_ONE);
			gl::ClipControl(gl::LOWER_LEFT, gl::ZERO_TO_ONE);
		}
	

		let event_pump = sdl.event_pump()?;
		let mut t = Self{ window_width, window_height, vsync: vsync, 
			_sdl: sdl, video, sdl_timer, window, event_pump, _gl_context };
		
		t.enable_vsync(vsync)?;
	
		return Ok(t);
	}

	pub fn enable_vsync(&mut self, enable_vsync: bool) -> Result<(), String>
	{
		if enable_vsync
		{
			self.video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync)?;
		}
		else
		{
			self.video.gl_set_swap_interval(sdl2::video::SwapInterval::Immediate)?;
		}

		self.vsync = enable_vsync;
		return Ok(());
	}

	pub fn run(&mut self) -> Result<(), String>
	{
		let box_size = 40;

		let vert_shader = render_gl::Shader::from_vert_source(
			&CString::new(include_str!("triangle.vert")).unwrap(), &"triangle.vert".to_string()
		)?;

		let frag_shader = render_gl::Shader::from_frag_source(
			&CString::new(include_str!("triangle.frag")).unwrap(), &"triangle.frag".to_string()
		)?;


		let shader_program = render_gl::Program::from_shaders(
			&[vert_shader, frag_shader]
		).unwrap();

		shader_program.set_used();

		let on_color = get_u32_agbr_color(1.0, 1.0, 1.0, 1.0);
		let off_color = get_u32_agbr_color(0.0, 0.0, 0.0, 1.0);

		let board_size_x = 8;
		let board_size_y = 12;

		let mut board: Vec<u32> = Vec::new();
		for _ in 0.. (board_size_x * board_size_y)
		{
			board.push(off_color);
		}

		let start_x: f32 = (-(board_size_x as f32) / 2.0f32 + 0.5f32) * box_size as f32;
		let start_y: f32 = (-(board_size_y as f32) / 2.0f32 + 0.5f32) * box_size as f32;

		let start_x_px = ((self.window_width - board_size_x * box_size) / 2) as i32;
		let start_y_px = ((self.window_height - board_size_y * box_size) / 2) as i32;
		let end_x_px = start_x_px + (board_size_x * box_size) as i32;
		let end_y_px = start_y_px + (board_size_y * box_size) as i32;

		let mut shader_data: Vec<ShaderData> = Vec::new();
		{
			let col = off_color;
			for y in 0..board_size_y
			{
				for x in 0..board_size_x
				{
					let pos_x = start_x + (x * box_size) as f32;
					let pos_y = start_y + (y * box_size) as f32; 

					shader_data.push(ShaderData{_pos_x: pos_x, _pos_y: pos_y, _col: col, _size: box_size as f32});
				}
			}
		}



		let ssbo: render_gl::ShaderBuffer = render_gl::ShaderBuffer::new_with_data(
			//gl::SHADER_STORAGE_BUFFER,
			gl::UNIFORM_BUFFER,
			shader_data.len() * std::mem::size_of::<ShaderData>(),
			shader_data.as_ptr() as *const gl::types::GLvoid
		);

		let mut vao: gl::types::GLuint = 0;
		unsafe
		{
			gl::GenVertexArrays(1, &mut vao);
		}

		let mut now_stamp: u64 = self.sdl_timer.performance_counter();
		let mut last_stamp: u64;
		let perf_freq: f64 = self.sdl_timer.performance_frequency() as f64;
		let mut _dt: f32;




		let mut mouse_x: i32 = 0;
		let mut mouse_y: i32 = 0;
		let mut mouse_b: i32 = 0;


		loop
		{

			last_stamp = now_stamp;
			now_stamp = self.sdl_timer.performance_counter();
			_dt = ((now_stamp - last_stamp) as f64 * 1000.0f64 / perf_freq ) as f32;


			for event in self.event_pump.poll_iter()
			{
				match event
				{
					Event::Quit {..} |
					Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
					{
						return Ok(());
					},

					Event::MouseButtonDown { mouse_btn, x, y, .. } =>
					{
						mouse_x = x;
						mouse_y = self.window_height as i32 - y;
						if mouse_btn == MouseButton::Left
						{
							mouse_b |= 1;
						}
						else if mouse_btn == MouseButton::Right
						{
							mouse_b |= 2;
						}
					},
					Event::MouseButtonUp { mouse_btn, x, y, .. } =>
					{
						mouse_x = x;
						mouse_y = self.window_height as i32 - y;
						if mouse_btn == MouseButton::Left
						{
							mouse_b &= !1;
						}
						else if mouse_btn == MouseButton::Right
						{
							mouse_b &= !2;
						}
					},
					Event::MouseMotion { x, y, .. } =>
					{
						mouse_x = x;
						mouse_y = self.window_height - y;
					},

					Event::Window {win_event, ..  } =>
					{
						match win_event
						{
							sdl2::event::WindowEvent::Resized( width, height ) =>
							{
								self.window_width = width;
								self.window_height = height;
								println!("Resized: {}: {}", self.window_width, self.window_height);
								unsafe
								{
									gl::Viewport(0, 0, self.window_width, self.window_height);
								}

							},

							_ => {}
						}
					},
					_ => {}
				}
			}

			if mouse_b != 0
			{
				if mouse_x >= start_x_px && mouse_y >= start_y_px
					&& mouse_x < end_x_px && mouse_y < end_y_px
				{
					let p_x = mouse_x - start_x_px;
					let p_y = mouse_y - start_y_px;

					let x_b = p_x / (box_size as i32);
					let y_b = p_y / (box_size as i32);

					let index = x_b + y_b * (board_size_x as i32);
					if mouse_b == 1 && index >= 0 && index < (board_size_x * board_size_y) as i32
					{
						board[index as usize] = on_color;
					}
					else if mouse_b == 2 && index >= 0 && index < (board_size_x * board_size_y) as i32
					{
						board[index as usize] = off_color;
					}
				} 
			}

			// Write all the tiles into color from background.
			for y in 0..board_size_y
			{
				for x in 0..board_size_x
				{
					let index = (y * board_size_x + x) as usize;
					shader_data[index]._col = board[index];
				}
			}


			ssbo.write_data(0, ssbo.get_size(), shader_data.as_ptr() as *const gl::types::GLvoid);




			shader_program.set_used();
			unsafe
			{
				gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
				gl::DepthFunc(gl::LESS);
				gl::Enable(gl::DEPTH_TEST);
				gl::DepthFunc(gl::ALWAYS);

				gl::Uniform4f(0, 0.0f32, box_size as f32, self.window_width as f32, self.window_height as f32);

				gl::BindVertexArray(vao);

				ssbo.bind(2);

				gl::DrawArrays(
					gl::TRIANGLES, // mode
					0, // starting index in the enabled arrays
					6 * shader_data.len() as i32 // number of indices to be rendered
				);

				//gl::DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_INT, std::ptr::null());
			}
			::std::thread::sleep(std::time::Duration::from_millis(1));
			self.window.gl_swap_window();
			//println!("x: {}, y: {}", pos_x, pos_y);
			//println!("Frame duration: {}", _dt);
		}
	}

	//return Ok(());
}

fn main()
{
	println!("Hello, world!");

	let mut app;
	match App::init(800, 600, "Paint", true)
	{
		Ok(v) => 
		{
			app = v;
			match app.run()
			{
				Ok(_) =>
				{

				}
				Err(f) =>
				{
					println!("Runtime error: {}", f);
					//panic!(f);
				}
			} 
		} 
		Err(e) => 
		{ 
			println!("Error: {}", e);
			//panic!(e);
		} 
	}
}