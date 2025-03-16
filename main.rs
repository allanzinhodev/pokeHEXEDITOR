use minifb::{Key, Window, WindowOptions};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const WIDTH: usize = 80;      // Colunas de caracteres
const HEIGHT: usize = 40;     // Linhas de caracteres
const BYTES_PER_ROW: usize = 16;

struct HexEditor {
    rom_path: Option<PathBuf>,
    data: Vec<u8>,
    modified: bool,
    view_offset: usize,
    cursor_pos: (usize, usize),  // Posição do cursor (x, y)
}

impl HexEditor {
    fn new() -> Self {
        Self {
            rom_path: None,
            data: Vec::new(),
            modified: false,
            view_offset: 0,
            cursor_pos: (0, 0),
        }
    }

    // Abrir um arquivo ROM
    fn open_file(&mut self, path: &str) -> io::Result<()> {
        let data = fs::read(path)?;
        self.data = data;
        self.rom_path = Some(PathBuf::from(path));
        self.view_offset = 0;
        self.modified = false;
        Ok(())
    }

    // Salvar as alterações no arquivo
    fn save_file(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.rom_path {
            fs::write(path, &self.data)?;
            self.modified = false;
            println!("Arquivo salvo: {}", path.display());
        }
        Ok(())
    }

    // Modificar um byte
    fn edit_byte(&mut self, offset: usize, value: u8) {
        if offset < self.data.len() {
            self.data[offset] = value;
            self.modified = true;
        }
    }

    // Calcular o offset absoluto baseado na posição do cursor
    fn get_cursor_offset(&self) -> Option<usize> {
        let (x, y) = self.cursor_pos;
        
        // Verificar se o cursor está na área de bytes (não no endereço ou ASCII)
        if x >= 10 && x < 10 + BYTES_PER_ROW * 3 && x % 3 != 2 {
            let byte_idx = (x - 10) / 3;
            let offset = self.view_offset + y * BYTES_PER_ROW + byte_idx;
            
            if offset < self.data.len() {
                return Some(offset);
            }
        }
        
        None
    }

    // Mover o cursor
    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let new_x = (self.cursor_pos.0 as isize + dx).max(0).min((10 + BYTES_PER_ROW * 3 - 1) as isize) as usize;
        let new_y = (self.cursor_pos.1 as isize + dy).max(0).min((HEIGHT - 1) as isize) as usize;
        
        self.cursor_pos = (new_x, new_y);
    }

    // Rolar a visualização
    fn scroll(&mut self, delta: isize) {
        if delta < 0 {
            self.view_offset = self.view_offset.saturating_sub(BYTES_PER_ROW);
        } else if delta > 0 {
            let max_offset = self.data.len().saturating_sub(HEIGHT * BYTES_PER_ROW);
            self.view_offset = (self.view_offset + BYTES_PER_ROW).min(max_offset);
        }
    }

    // Renderizar o conteúdo do editor para o terminal
    fn render(&self) {
        // Limpar tela
        print!("\x1B[2J\x1B[1;1H");
        
        // Cabeçalho
        println!("=== Editor Hexadecimal para ROMs de Pokémon ===");
        if let Some(ref path) = self.rom_path {
            println!("Arquivo: {} ({}{})", 
                    path.display(), 
                    self.data.len(), 
                    if self.modified { ", modificado" } else { "" });
        } else {
            println!("Nenhum arquivo aberto");
        }
        println!("-------------------------------------------------------------------------------");
        
        // Instruções
        println!("Comandos: Setas (mover), PgUp/PgDn (rolar), Enter (editar), S (salvar), O (abrir), Q (sair)");
        println!("-------------------------------------------------------------------------------");
        
        if self.data.is_empty() {
            println!("Nenhum dado para exibir. Use 'O' para abrir um arquivo.");
            return;
        }
        
        // Cabeçalho da tabela
        print!("Offset    | ");
        for i in 0..BYTES_PER_ROW {
            print!("{:02X} ", i);
        }
        println!("| ASCII");
        println!("-----------+-------------------------------------------------+-----------------");
        
        // Linhas de dados
        let visible_rows = HEIGHT - 10; // Ajustar para as linhas de cabeçalho
        let end_offset = std::cmp::min(
            self.view_offset + visible_rows * BYTES_PER_ROW,
            self.data.len()
        );
        
        let mut row_offset = self.view_offset;
        let mut display_row = 0;
        
        while row_offset < end_offset {
            let row_end = std::cmp::min(row_offset + BYTES_PER_ROW, self.data.len());
            
            // Endereço
            print!("0x{:08X} |", row_offset);
            
            // Bytes em hexadecimal
            for i in row_offset..row_end {
                let is_cursor_here = self.cursor_pos == ((i - row_offset) * 3 + 10, display_row);
                
                if is_cursor_here {
                    print!(" \x1B[7m{:02X}\x1B[0m", self.data[i]); // Inverter cores para o cursor
                } else {
                    print!(" {:02X}", self.data[i]);
                }
            }
            
            // Preencher espaços vazios
            for _ in row_end..row_offset + BYTES_PER_ROW {
                print!("   ");
            }
            
            // ASCII
            print!(" | ");
            for i in row_offset..row_end {
                let byte = self.data[i];
                if byte >= 32 && byte <= 126 {
                    print!("{}", byte as char);
                } else {
                    print!(".");
                }
            }
            println!();
            
            row_offset += BYTES_PER_ROW;
            display_row += 1;
        }
    }
}

fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Falha ao ler entrada");
    input.trim().to_string()
}

fn main() {
    let mut editor = HexEditor::new();
    
    // Verificar argumentos de linha de comando
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match editor.open_file(&args[1]) {
            Ok(_) => println!("Arquivo aberto: {}", args[1]),
            Err(e) => println!("Erro ao abrir arquivo: {}", e),
        }
    }
    
    // Configurar janela
    let mut window = Window::new(
        "Editor Hexadecimal para ROMs de Pokémon",
        WIDTH * 10, HEIGHT * 20,  // Tamanho aproximado da janela
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    
    // Limitar FPS para não consumir muita CPU
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));
    
    // Loop principal
    while window.is_open() && !window.is_key_down(Key::Q) {
        // Entrada do teclado
        if window.is_key_released(Key::O) {
            // Abrir arquivo
            let filename = get_input("Digite o caminho do arquivo para abrir: ");
            if !filename.is_empty() {
                match editor.open_file(&filename) {
                    Ok(_) => println!("Arquivo aberto: {}", filename),
                    Err(e) => println!("Erro ao abrir arquivo: {}", e),
                }
            }
        }
        
        if window.is_key_released(Key::S) {
            // Salvar arquivo
            if editor.rom_path.is_none() {
                let filename = get_input("Digite o caminho para salvar: ");
                if !filename.is_empty() {
                    editor.rom_path = Some(PathBuf::from(&filename));
                }
            }
            
            if let Err(e) = editor.save_file() {
                println!("Erro ao salvar: {}", e);
            }
        }
        
        // Movimentação do cursor
        if window.is_key_released(Key::Up) {
            editor.move_cursor(0, -1);
        }
        if window.is_key_released(Key::Down) {
            editor.move_cursor(0, 1);
        }
        if window.is_key_released(Key::Left) {
            editor.move_cursor(-3, 0);  // 3 caracteres por byte (2 dígitos + espaço)
        }
        if window.is_key_released(Key::Right) {
            editor.move_cursor(3, 0);
        }
        
        // Rolagem
        if window.is_key_released(Key::PageUp) {
            editor.scroll(-10);
        }
        if window.is_key_released(Key::PageDown) {
            editor.scroll(10);
        }
        
        // Edição de bytes
        if window.is_key_released(Key::Enter) {
            if let Some(offset) = editor.get_cursor_offset() {
                let current_value = editor.data[offset];
                let input = get_input(&format!("Editar byte em 0x{:08X} [valor atual: 0x{:02X}]: 0x", offset, current_value));
                
                if !input.is_empty() {
                    if let Ok(value) = u8::from_str_radix(&input, 16) {
                        editor.edit_byte(offset, value);
                        println!("Byte 0x{:08X} alterado para 0x{:02X}", offset, value);
                    } else {
                        println!("Valor inválido. Use formato hexadecimal (ex: 1F)");
                    }
                }
            }
        }
        
        // Renderizar a interface
        editor.render();
        
        // Atualizar a janela
        window.update();
    }
}