use anyhow::{bail, Result};
use obws::Client;
use std::env;
use std::time::Duration;
use streamdeck::{pids, DeviceImage, Error, ImageOptions, StreamDeck};

async fn get_state(obs: &Client) -> Result<State> {
    let current_scene = obs.scenes().current_program_scene().await?;
    let recording = obs.recording().status().await?;
    Ok(State {
        current_scene,
        recording: recording.active,
    })
}

fn render(state: &State, icons: &Icons, deck: &mut StreamDeck) -> Result<()> {
    match state.current_scene.as_str() {
        "Coding" => deck.write_button_image(1, &icons.helix)?,
        "Talking" => deck.write_button_image(1, &icons.talking)?,
        &_ => (),
    };

    match state.recording {
        true => deck.write_button_image(2, &icons.start_recording)?,
        false => deck.write_button_image(2, &icons.stop_recording)?,
    }
    Ok(())
}

struct State {
    current_scene: String,
    recording: bool,
}

struct Icons {
    helix: DeviceImage,
    talking: DeviceImage,
    start_recording: DeviceImage,
    stop_recording: DeviceImage,
}

fn load_icons(deck: &mut StreamDeck) -> Result<Icons> {
    let helix = deck.load_image("./icons/helix.png", &ImageOptions::default())?;
    let talking = deck.load_image("./icons/talking.png", &ImageOptions::default())?;
    let start_recording =
        deck.load_image("./icons/start-recording.png", &ImageOptions::default())?;
    let stop_recording = deck.load_image("./icons/stop-recording.png", &ImageOptions::default())?;
    Ok(Icons {
        helix,
        talking,
        start_recording,
        stop_recording,
    })
}

async fn handle_top_left(old_state: &State, obs: &Client) -> Result<()> {
    match old_state.current_scene.as_str() {
        "Coding" => obs.scenes().set_current_program_scene("Talking").await?,
        "Talking" => obs.scenes().set_current_program_scene("Coding").await?,
        &_ => bail!("Unexpected scene"),
    }
    Ok(())
}

async fn handle_top_middle(old_state: &State, obs: &Client) -> Result<()> {
    if old_state.recording {
        obs.recording().stop().await?;
    } else {
        obs.recording().start().await?;
    }
    Ok(())
}

async fn handle_press(obs: &Client, pressed: &Vec<u8>, state: &State) -> Result<()> {
    if pressed[0] == 1 {
        handle_top_left(&state, obs).await?;
    }
    if pressed[1] == 1 {
        handle_top_middle(&state, obs).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let obs_passwd = env::var("OBS_PASSWD").map_or(None, |pwd| Some(pwd));

    let client = Client::connect("localhost", 4455, obs_passwd).await?;

    const ELGATO_VID: u16 = 0x0fd9;
    let mut deck = StreamDeck::connect(ELGATO_VID, pids::REVISED_MINI, None)?;

    const POLL_WAIT: Duration = Duration::new(1, 0);

    let icons = load_icons(&mut deck)?;

    loop {
        let state = get_state(&client).await?;
        render(&state, &icons, &mut deck)?;
        let result = deck.read_buttons(Some(POLL_WAIT));
        match result {
            Ok(pressed) => handle_press(&client, &pressed, &state).await?,
            Err(err) => {
                if !matches!(err, Error::NoData) {
                    bail!(err)
                }
            }
        }
    }
}
