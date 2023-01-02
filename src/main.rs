use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, 
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    ExecutableCommand
};
use rusty_audio::Audio;
use std::{
    error::Error,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
    {io, thread},
};

use mpsc::*;
use invaders::{frame::{self, new_frame, Drawable, Frame}, invaders::Invaders};
use invaders::render::{self};
use invaders::player::Player;

fn main() -> Result <(), Box<dyn Error>> {
    // Inicia a library para execução de audio
    let mut audio = Audio::new();

    // Carrega os arquivos na pasta do projeto associando um nome a cada um deles
    audio.add("explode", "sounds/explode.wav");
    audio.add("lose", "sounds/lose.wav");
    audio.add("move", "sounds/move.wav");
    audio.add("pew", "sounds/pew.wav");
    audio.add("startup", "sounds/startup.wav");
    audio.add("win", "sounds/win.wav");

    // Executa o audio 'startup'
    audio.play("startup");

    let mut stdout = io::stdout();

    // Captura as teclas acionadas
    terminal::enable_raw_mode()?;

    // Executa o script em uma nova tela no terminal
    stdout.execute(EnterAlternateScreen)?;
    
    // Esconde o cursor do terminal
    stdout.execute(Hide)?;

    // Renderizando

    // Canais de comunicação com as threads de renderização
    let (render_tx, render_rx) = mpsc::channel();
    
    // Iniciando a thread
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };

            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Iniciando elementos do jogo(jogador, contador de tempo e invasores)
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    // Loop principal que só será interrompido na derrota ou vitória
    'gameloop: loop {
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        while event::poll(Duration::default())? {
            // Detecta uma tecla pressionada pelo usuário e executa uma ação
            if let Event::Key(key_event) = event::read()? {
                 match key_event.code {
                     // Finaliza o jogo se for esc ou q
                     KeyCode::Esc | KeyCode::Char('q') => {
                         audio.play("lose");
                         break 'gameloop;
                     }
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    _ => {}
                }
            }
        }

        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }

        player.draw(&mut curr_frame);
        invaders.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        // Vitória ou derrota
        if invaders.all_killed() {
            audio.play("win");
            break 'gameloop;
        }
        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }

    drop(render_tx);
    render_handle.join().unwrap();

    // Aguarda a execução de todos os áudios finalizar para continuar a execução
    audio.wait();
    
    // Volta a exibir o cursor
    stdout.execute(Show)?;
    
    // Sai da tela secundária do terminal, voltando a tela de onde o programa foi iniciado
    stdout.execute(LeaveAlternateScreen)?;

    terminal::disable_raw_mode();

    Ok(())
}
