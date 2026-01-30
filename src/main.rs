#[macro_use]
extern crate lazy_static;

pub mod cpu;
pub mod nes;

use std::{
    io,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, layout::{Constraint, Layout, Margin, Rect}, style::Stylize, symbols::border, text::Line, widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState}
};
use cpu::base::Processor;
use nes::Nes;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // let args: Vec<String> = env::args().collect();
    // let filepath = &args[1];

    let cpu = Processor::new(None);
    let mut nes = Nes::new(cpu);
    // nes.load_cartridge(filepath);
    nes.load_cartridge(&"nestest.nes".to_string());
    nes.reset(Some(0x0C000));
    let mut app = App::new(&mut nes);

    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}

#[derive(Debug)]
pub struct App<'a> {
    nes: &'a mut Nes,
    low_pc: usize,
    high_pc: usize,
    ram_scroll_state: ScrollbarState,
    ram_vertical_scroll: usize,
    available_ram_lines: usize,
    exit: bool,
}

impl<'a> App<'a> {
    pub fn new(nes: &'a mut Nes) -> Self {
        App {
            nes,
            low_pc: 0,
            high_pc: 0,
            exit: false,
            ram_vertical_scroll: 0,
            ram_scroll_state: ScrollbarState::new(0),
            available_ram_lines: 0,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        use Constraint::{Fill, Length, Min, Percentage};

        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [title_area, main_area, status_area] = vertical.areas(frame.area());
        let horizontal = Layout::horizontal([Fill(1); 2]);
        let [left_area, right_area] = horizontal.areas(main_area);
        let vertical2 = Layout::vertical([Percentage(30), Percentage(70)]);
        let [flags_area, memory_area] = vertical2.areas(right_area);

        frame.render_widget(Block::bordered().title("Nes"), title_area);
        frame.render_widget(Block::bordered().title("<Q> to quit, <T> to test"), status_area);
        // frame.render_widget(self.render_program(), left_area);
        // frame.render_widget(Block::bordered().title("Memory"), memory_area);

        self.render_flags(frame, flags_area);
        self.render_program(frame, left_area);
        self.render_ram(frame, memory_area);
        // frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('t') => self.load_nes_test(),
            KeyCode::Char('r') => self.run_cpu(),
            KeyCode::Down => {
                if self.ram_vertical_scroll < self.available_ram_lines as usize {
                    self.ram_vertical_scroll = self.ram_vertical_scroll.saturating_add(8);
                    self.ram_scroll_state = self.ram_scroll_state.position(self.ram_vertical_scroll);
                }
            },
            KeyCode::Up => {
                self.ram_vertical_scroll = self.ram_vertical_scroll.saturating_sub(8);
                self.ram_scroll_state = self.ram_scroll_state.position(self.ram_vertical_scroll);
            },
            _ => {}
        };
    }

    fn load_nes_test(&mut self) {
        self.nes.load_cartridge(&"nestest.nes".to_string());
        self.nes.reset(Some(0x0C000));
    }

    fn run_cpu(&mut self) {
        self.nes.cpu.exec();
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render_program(&mut self, f: &mut Frame, rect: Rect)  {
        let pc = self.nes.cpu.state.pc;
        if pc > self.high_pc || pc < self.low_pc {
            self.low_pc = pc;
        }

        let mut lines : Vec<Line> = vec![];

        let mut text: String;
        let mut loc: usize = self.low_pc;

        for _i in 0..32 {
           self.high_pc = loc;
           (text, loc, _) = self.nes.cpu.debug_opcode(loc);
           if self.high_pc == pc {
               lines.push(Line::from(text).green().bold().underlined());
           } else {
               lines.push(Line::from(text));
           }
        }


        let block = Block::bordered()
            .title(Line::raw("Program").centered())
            .border_set(border::PLAIN);

        f.render_widget(
            Paragraph::new(lines).block(block),
            rect
        )
    }

    fn render_ram(&mut self, f: &mut Frame, rect: Rect) {
        let mut lines: Vec<Line> = vec![];

        let w: u32 = (rect.width - 8) as u32 / 3;
        self.available_ram_lines = 0x800 / w as usize;
        self.ram_scroll_state = self.ram_scroll_state.content_length(self.available_ram_lines);

        for i in 0..self.available_ram_lines {
            let base = i * w as usize;
            let mut values : Vec<String> = vec![
                format!("{:04X}", base)
            ];
            for a in 0..w {
                values.push(format!("{:02X}", self.nes.cpu.mem.read(base + a as usize)));
            }
            lines.push(Line::from(values.join(" ")));
        }

        let block = Block::bordered()
            .title(Line::raw("Memory").centered())
            .border_set(border::PLAIN);

        f.render_widget(Paragraph::new(lines).scroll((self.ram_vertical_scroll as u16, 0)).block(block), rect);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

        f.render_stateful_widget(scrollbar, rect.inner(Margin {vertical: 1, horizontal: 0}), &mut self.ram_scroll_state);
    }

    fn render_flags(&self, f: &mut Frame, rect: Rect) {
        let bits: Vec<String> = (0..8)
            .rev()
            .map(|i| ((self.nes.cpu.state.status >> i) & 1).to_string())
            .collect();

        let text = vec![
            "N V 1 B D I Z C".into(),
            Line::from(bits.join(" ")),
            Line::from(format!("A: {:02X} X: {:02X} Y: {:02X} PC: {:04X}", self.nes.cpu.state.a, self.nes.cpu.state.x, self.nes.cpu.state.y, self.nes.cpu.state.pc))
        ];
        let block = Block::bordered()
            .title(Line::raw("Flags").centered())
            .border_set(border::PLAIN);
        f.render_widget(
            Paragraph::new(text).block(block),
            rect
        )
    }

}

