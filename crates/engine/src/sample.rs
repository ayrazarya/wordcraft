//! Built-in sample documents and templates (original text, generated in code).

use wordcraft_doc::para::InlineObject;
use wordcraft_doc::props::{Align, Border, BorderStyle, Borders, CharProps, NumRef, Rgb, TabAlign, TabLeader, TabStop, TextColor};
use wordcraft_doc::{Block, Document, ListKind, Paragraph, PartKind, Table, para_block};

fn para(text: &str) -> Paragraph {
    Paragraph::with_text(text, CharProps::default())
}

fn styled(text: &str, style: &str) -> Paragraph {
    para(text).styled(style)
}

/// Append text with formatting to a paragraph.
fn add(p: &mut Paragraph, text: &str, props: CharProps) {
    let at = p.len();
    let _ = p.insert_text(at, text, &props);
}

fn bold() -> CharProps {
    CharProps { bold: Some(true), ..Default::default() }
}
fn italic() -> CharProps {
    CharProps { italic: Some(true), ..Default::default() }
}

/// The showcase document: a short studio handbook with headings, lists, a table, a quote,
/// a footer with page numbers.
pub fn sample_document() -> Document {
    let mut d = Document::new();
    d.core.title = "The Open Studio Handbook".into();
    d.core.creator = "WordCraft".into();
    let mut b: Vec<Block> = Vec::new();
    b.push(Block::Para(styled("The Open Studio Handbook", "Title")));
    b.push(Block::Para(styled("How a small community of makers shares space, tools and time", "Subtitle")));
    let mut intro = para("Every Thursday evening the old print shop on Alder Street fills with people who make things. ");
    add(&mut intro, "Painters", bold());
    add(&mut intro, " set up by the north windows, ", CharProps::default());
    add(&mut intro, "letterpress", italic());
    add(
        &mut intro,
        " regulars claim the back room, and somebody always brings bread. This handbook collects what we have learned about running a shared studio: the routines that keep it pleasant, the tools we pool, and the small agreements that let thirty strangers become a crew.",
        CharProps::default(),
    );
    b.push(Block::Para(intro));
    b.push(Block::Para(styled("Why share a studio?", "Heading1")));
    b.push(Block::Para(para("Rent is the obvious reason, but it is not the best one. A shared room puts your work next to work you would never have seen otherwise. A ceramicist explains glaze chemistry to a comic artist; a photographer borrows a weaver's loom for a still life. The cross-talk is the point.")));
    let mut quote = styled("A studio is a place where unfinished things are welcome.", "Quote");
    quote.props.space_before = Some(12.0);
    b.push(Block::Para(quote));
    b.push(Block::Para(styled("What we agree on", "Heading2")));
    let mut num = d.numbering.clone();
    let bullets = num.add_list(ListKind::Bullet);
    let steps = num.add_list(ListKind::Numbered);
    d.numbering = num;
    for (lead, rest) in [
        ("Clean as you go. ", "Leave a table the way you would like to find it."),
        ("Label what is yours. ", "Unlabelled supplies on the commons shelf are fair game."),
        ("Ask before you borrow. ", "Even from the shelf, a quick note helps everyone keep track."),
        ("Quiet hours ", "run from 9 to 11 on weekday mornings."),
    ] {
        let mut p = Paragraph::new().styled("ListParagraph");
        add(&mut p, lead, bold());
        add(&mut p, rest, CharProps::default());
        p.props.numbering = Some(NumRef { num: bullets, level: 0 });
        b.push(Block::Para(p));
    }
    b.push(Block::Para(styled("Shared equipment", "Heading2")));
    b.push(Block::Para(para("The table below lists what the membership owns together and who keeps each item in working order.")));
    let rows = [
        ["Equipment", "Location", "Steward", "Booking"],
        ["Etching press", "Back room", "Mara O.", "Sign-up sheet"],
        ["Kiln (cone 10)", "Yard shed", "Theo P.", "48 h notice"],
        ["Large-format printer", "Office", "Ines R.", "Calendar"],
        ["Light table", "North windows", "Anyone", "First come"],
    ];
    let mut t = Table::new(rows.len(), 4, 468.0);
    t.props.style = Some("GridTable4AccentBlue".into());
    for (r, row) in rows.iter().enumerate() {
        for (c, txt) in row.iter().enumerate() {
            if let Some(cell) = t.rows.get_mut(r).and_then(|x| x.cells.get_mut(c)) {
                cell.blocks = vec![para_block(para(txt))];
            }
        }
    }
    if let Some(r) = t.rows.first_mut() {
        r.props.header = true;
    }
    b.push(Block::Table(t));
    b.push(Block::Para(styled("Opening the studio", "Heading2")));
    for s in [
        "Switch on the main breaker by the door.",
        "Open both skylights unless it is raining.",
        "Start the kettle. This step is not optional.",
        "Check the booking sheet for the kiln and press.",
    ] {
        let mut p = para(s).styled("ListParagraph");
        p.props.numbering = Some(NumRef { num: steps, level: 0 });
        b.push(Block::Para(p));
    }
    b.push(Block::Para(styled("Membership", "Heading1")));
    let mut m = para("Membership is ");
    add(&mut m, "pay what you can", CharProps { bold: Some(true), color: Some(TextColor::Rgb(Rgb(0x15, 0x60, 0x82))), ..Default::default() });
    add(
        &mut m,
        ", reviewed every season. New members shadow a regular for their first two visits, and everyone takes one cleaning shift a month. If you have never made anything before, you are exactly who we hoped would show up.",
        CharProps::default(),
    );
    b.push(Block::Para(m));
    let mut h = para("Questions go to the front desk, or to the notice board by the sink. ");
    add(
        &mut h,
        "Visit the community board",
        CharProps { style: Some("Hyperlink".into()), link: Some("https://getartcraft.com/".into()), ..Default::default() },
    );
    add(&mut h, " for events.", CharProps::default());
    b.push(Block::Para(h));
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    // Footer: centred page number.
    let mut f = Paragraph::new().styled("Footer");
    f.props.align = Some(Align::Center);
    let _ = f.insert_object(0, InlineObject::Field { instr: "PAGE".into(), result: "1".into(), locked: false, code: false }, &CharProps::default());
    let fid = d.add_part(PartKind::Footer, vec![para_block(f)]);
    d.last_section.footers.default = Some(fid);
    let mut hd = styled("The Open Studio Handbook\t\tSpring edition", "Header");
    hd.props.borders = Some(Borders {
        bottom: Some(Border { style: BorderStyle::Single, width: 0.5, color: Some(Rgb(0xBF, 0xBF, 0xBF)), space: 4.0 }),
        ..Default::default()
    });
    let hid = d.add_part(PartKind::Header, vec![para_block(hd)]);
    d.last_section.headers.default = Some(hid);
    d
}

/// A block-style letter.
pub fn letter() -> Document {
    let mut d = Document::new();
    let lines: Vec<Paragraph> = vec![
        styled("Your Name", "Title"),
        para("123 Your Street · City, ST 00000 · you@example.com"),
        para(""),
        para(&crate::cmd::insert::format_date("MMMM d, yyyy")),
        para(""),
        para("Recipient Name"),
        para("Title, Company"),
        para("Street Address"),
        para(""),
        para("Dear Recipient,"),
        para(
            "Start with a sentence that says why you are writing. Keep paragraphs short and specific, and close with what you would like to happen next.",
        ),
        para("Thank you for your time."),
        para("Sincerely,"),
        para(""),
        para("Your Name"),
    ];
    d.body = lines.into_iter().map(para_block).collect();
    d
}

/// A one-page résumé.
pub fn resume() -> Document {
    let mut d = Document::new();
    let mut b: Vec<Block> =
        vec![Block::Para(styled("Alex Rivera", "Title")), Block::Para(para("Illustrator · alex@example.com · portfolio.example.com"))];
    for (head, items) in [
        ("Experience", vec![("Lead Illustrator, Harbor Books", "2021 – present"), ("Freelance Illustrator", "2016 – 2021")]),
        ("Education", vec![("BFA Illustration, Coastal College of Art", "2016")]),
        ("Skills", vec![("Ink, gouache, digital painting, book layout, lettering", "")]),
    ] {
        b.push(Block::Para(styled(head, "Heading1")));
        for (what, when) in items {
            let mut p = para(what);
            p.props.tabs = Some(vec![TabStop { pos: 468.0, align: TabAlign::Right, leader: TabLeader::None }]);
            if !when.is_empty() {
                add(&mut p, &format!("\t{when}"), italic());
            }
            b.push(Block::Para(p));
        }
    }
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// A report with a cover heading and sections.
pub fn report() -> Document {
    let mut d = Document::new();
    let b: Vec<Block> = vec![
        Block::Para(styled("Report Title", "Title")),
        Block::Para(styled("Subtitle or date", "Subtitle")),
        Block::Para(styled("Summary", "Heading1")),
        Block::Para(para("Summarise the findings in two or three sentences.")),
        Block::Para(styled("Background", "Heading1")),
        Block::Para(para("Describe the context and the question the report answers.")),
        Block::Para(styled("Findings", "Heading1")),
        Block::Para(styled("First finding", "Heading2")),
        Block::Para(para("Explain the evidence.")),
        Block::Para(styled("Recommendations", "Heading1")),
        Block::Para(para("List what should happen next.")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// Indonesian academic thesis (Skripsi / Tugas Akhir) with standard 4-4-3-3 cm margins.
pub fn skripsi() -> Document {
    let mut d = Document::new();
    d.core.title = "Draf Skripsi".into();
    d.core.creator = "WordCraft".into();
    // Standard Indonesian academic margins: 4cm Left, 4cm Top, 3cm Right, 3cm Bottom on A4
    d.last_section.page_w = 595.3;
    d.last_section.page_h = 841.9;
    d.last_section.margin_top = wordcraft_geom::cm_to_pt(4.0);
    d.last_section.margin_left = wordcraft_geom::cm_to_pt(4.0);
    d.last_section.margin_bottom = wordcraft_geom::cm_to_pt(3.0);
    d.last_section.margin_right = wordcraft_geom::cm_to_pt(3.0);

    let b: Vec<Block> = vec![
        Block::Para(styled("JUDUL PENELITIAN SKRIPSI ATAU TUGAS AKHIR", "Title")),
        Block::Para(styled("Diajukan Untuk Memenuhi Sebagian Persyaratan Mencapai Derajat Sarjana", "Subtitle")),
        Block::Para(para("Nama Mahasiswa (NIM: 1234567890)")),
        Block::Para(para("Program Studi Teknik Informatika · Fakultas Ilmu Komputer · Universitas 2026")),
        Block::Para(styled("ABSTRAK", "Heading1")),
        Block::Para(para(
            "Penelitian ini bertujuan untuk menganalisis dan merancang solusi berbasis teknologi modern. Metode yang digunakan adalah kualitatif dan eksperimental. Hasil penelitian menunjukkan peningkatan efisiensi yang signifikan.",
        )),
        Block::Para(styled("KATA PENGANTAR", "Heading1")),
        Block::Para(para(
            "Puji dan syukur penulis panjatkan ke hadirat Tuhan Yang Maha Esa atas rahmat dan karunia-Nya sehingga skripsi ini dapat diselesaikan dengan baik.",
        )),
        Block::Para(styled("BAB I PENDAHULUAN", "Heading1")),
        Block::Para(styled("1.1 Latar Belakang Masalah", "Heading2")),
        Block::Para(para(
            "Perkembangan ilmu pengetahuan dan teknologi saat ini menuntut adanya inovasi yang berkelanjutan. Dalam bidang pengolahan dokumen, kecepatan dan keandalan menjadi faktor kunci.",
        )),
        Block::Para(styled("1.2 Rumusan Masalah", "Heading2")),
        Block::Para(para(
            "Berdasarkan latar belakang tersebut, rumusan masalah dalam penelitian ini adalah bagaimana membangun sistem pengolah kata yang cepat, aman, dan efisien.",
        )),
        Block::Para(styled("1.3 Tujuan Penelitian", "Heading2")),
        Block::Para(para("Tujuan dari penelitian ini adalah menghasilkan perangkat lunak yang andal untuk kebutuhan akademis dan profesional.")),
        Block::Para(styled("1.4 Manfaat Penelitian", "Heading2")),
        Block::Para(para(
            "Manfaat penelitian ini mencakup kontribusi teoritis bagi literatur akademik serta manfaat praktis bagi masyarakat pengguna.",
        )),
        Block::Para(styled("BAB II TINJAUAN PUSTAKA", "Heading1")),
        Block::Para(styled("2.1 Landasan Teori", "Heading2")),
        Block::Para(para("Bagian ini memaparkan teori-teori dasar dan kerangka pemikiran yang relevan dengan topik yang diteliti.")),
        Block::Para(styled("BAB III METODOLOGI PENELITIAN", "Heading1")),
        Block::Para(styled("3.1 Metode Penelitian", "Heading2")),
        Block::Para(para("Penelitian ini menggunakan pendekatan clean-room engineering dengan pengujian komprehensif.")),
        Block::Para(styled("DAFTAR PUSTAKA", "Heading1")),
        Block::Para(para("Santoso, B. (2025). Rekayasa Perangkat Lunak Modern. Jakarta: Penerbit Informatika.")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// Indonesian official letter (Surat Resmi / Dinas) template with letterhead and standard structure.
pub fn surat_resmi() -> Document {
    let mut d = Document::new();
    d.core.title = "Surat Resmi".into();
    d.last_section.page_w = 595.3;
    d.last_section.page_h = 841.9;
    d.last_section.margin_top = wordcraft_geom::cm_to_pt(2.5);
    d.last_section.margin_left = wordcraft_geom::cm_to_pt(3.0);
    d.last_section.margin_bottom = wordcraft_geom::cm_to_pt(2.5);
    d.last_section.margin_right = wordcraft_geom::cm_to_pt(2.5);

    let mut kop = styled("PEMERINTAH KOTA / INSTANSI RESMI", "Title");
    kop.props.align = Some(Align::Center);
    let mut sub_kop = para("Jl. Jenderal Sudirman No. 123 · Telp. (021) 1234567 · surel: info@instansi.go.id");
    sub_kop.props.align = Some(Align::Center);
    sub_kop.props.borders = Some(Borders {
        bottom: Some(Border { style: BorderStyle::Single, width: 1.5, color: Some(Rgb(0x22, 0x22, 0x22)), space: 6.0 }),
        ..Default::default()
    });

    let b: Vec<Block> = vec![
        Block::Para(kop),
        Block::Para(sub_kop),
        Block::Para(para("")),
        Block::Para(para("Nomor       : 001/UND/X/2026\t\t\tJakarta, 13 Oktober 2026")),
        Block::Para(para("Lampiran  : -")),
        Block::Para(para("Perihal     : Undangan Rapat Koordinasi")),
        Block::Para(para("")),
        Block::Para(para("Kepada Yth.")),
        Block::Para(para("Bapak/Ibu Pimpinan Unit Kerja")),
        Block::Para(para("di Tempat")),
        Block::Para(para("")),
        Block::Para(para("Dengan hormat,")),
        Block::Para(para(
            "Sehubungan dengan pelaksanaan program kerja tahun 2026, kami mengundang Bapak/Ibu untuk menghadiri rapat koordinasi yang akan dilaksanakan pada:",
        )),
        Block::Para(para("    Hari/Tanggal : Senin, 20 Oktober 2026")),
        Block::Para(para("    Waktu        : 09.00 WIB s.d. selesai")),
        Block::Para(para("    Tempat       : Ruang Rapat Utama Lantai 2")),
        Block::Para(para("Mengingat pentingnya agenda tersebut, kami mengharapkan kehadiran tepat pada waktunya.")),
        Block::Para(para("Demikian surat undangan ini kami sampaikan. Atas perhatian dan kerja sama Bapak/Ibu, kami ucapkan terima kasih.")),
        Block::Para(para("")),
        Block::Para(para("Hormat kami,")),
        Block::Para(para("Kepala Instansi / Pimpinan,")),
        Block::Para(para("")),
        Block::Para(para("")),
        Block::Para(styled("(Nama Pejabat / Pimpinan)", "Subtitle")),
        Block::Para(para("NIP. 19850101 201001 1 001")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_are_valid() {
        assert_eq!(sample_document().word_count(), 361);
        for d in [sample_document(), letter(), resume(), report(), skripsi(), surat_resmi()] {
            assert!(d.word_count() > 5);
            let l = wordcraft_layout::layout(&d, &mut wordcraft_layout::LayoutCache::new(), &Default::default());
            assert!(!l.pages.is_empty());
        }
    }
}
