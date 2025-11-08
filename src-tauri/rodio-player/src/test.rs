#[cfg(test)]
mod tests {
    use crate::decoder::Decoder;
    use std::{ffi::CString, fs::File, io::Write, sync::Arc, thread, time::Duration};

    #[test]
    fn test_decoder() {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        let path =
            "https://rr2---sn-n4v7snly.googlevideo.com/videoplayback?expire=1762355028&ei=9BILab_EI_CSsfIP9Njs6QU&ip=104.3.190.153&id=o-ANqSKQPze4otmtCqBsg36e4sShLFWpFk3em_o4FsdVVi&itag=251&source=youtube&requiressl=yes&xpc=EgVo2aDSNQ%3D%3D&met=1762333428%2C&mh=4I&mm=31%2C26&mn=sn-n4v7snly%2Csn-a5msen76&ms=au%2Conr&mv=m&mvi=2&pl=21&rms=au%2Cau&gcr=us&initcwndbps=2263750&bui=AdEuB5SuPzw6X5GwWbbq-sB4fKfO-OXU4dtDY6N2F3KThhihSWG4ynDn6dPB_vAEg5Jgb0FLkKIY1Lbc&vprv=1&svpuc=1&mime=audio%2Fwebm&ns=ZqkW5Pdb7QHeXceHIKsvkRoQ&rqh=1&gir=yes&clen=3101071&dur=200.941&lmt=1727211552505889&mt=1762333053&fvip=5&keepalive=yes&lmw=1&fexp=51557447%2C51565116%2C51565682%2C51580970&c=TVHTML5&sefc=1&txp=4532434&n=3naMyj8xGAg_Zg&sparams=expire%2Cei%2Cip%2Cid%2Citag%2Csource%2Crequiressl%2Cxpc%2Cgcr%2Cbui%2Cvprv%2Csvpuc%2Cmime%2Cns%2Crqh%2Cgir%2Cclen%2Cdur%2Clmt&lsparams=met%2Cmh%2Cmm%2Cmn%2Cms%2Cmv%2Cmvi%2Cpl%2Crms%2Cinitcwndbps&lsig=APaTxxMwRAIgeZFz9jV09S9pG8_tlcfZypbrVNHsYLHIUMS53p7bJJICIH-KjLFPUWk1_49RjCsS0kYXL3dzaIiIS9zcwFSyXTb2&sig=AJfQdSswRQIhAOa9A0pfA39hUn9CJ5190aJzlSPv1HOeG5eco9eoETHDAiB98EQ2TAJit7eW9nXdd1NLyO_0pRZFQX-xqH24UBD_CQ%3D%3D";
        let decoder = Decoder::open(path).unwrap();

        // let mut out = File::create("./out.mp3").unwrap();
        // for frame in decoder {
        //     out.write_all(&frame.to_le_bytes());
        // }
        sink.append(decoder);
        sink.set_volume(1f32);
        sink.play();

        let sink_arc = Arc::new(sink);
        thread::spawn(move || {
            let sink = sink_arc.clone();
            loop {
                println!("Seeking");
                sink.try_seek(Duration::from_secs(30)).unwrap();
                thread::sleep(Duration::from_secs(10));
            }
        })
        .join()
        .unwrap();
    }
}

/*
Inspecting frame: format=8 planar=true channels=2 samples=648
ch 0: f32: mean=-0.000289 min=-0.184784 max=0.172244 first10=[0.0017771054, 0.013189024, 0.024515674, 0.03141686, 0.029912189, 0.018975744, 0.00020472656, -0.022409761, -0.038968828, -0.044706207]
ch 1: f32: mean=-0.001101 min=-0.154588 max=0.137305 first10=[0.032576278, 0.024091328, 0.014422949, 0.009586434, 0.009024219, 0.009087507, 0.009660266, 0.011900065, 0.017822402, 0.025243182]
*/