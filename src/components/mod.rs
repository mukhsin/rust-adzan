//! Komponen komposabel: potongan UI yang dirakit menjadi layar.
//!
//! - [`title_bar`] — bar atas window utama (brand + tab + kontrol jendela).
//! - [`widget`] — root kartu tiket di widget window.
//! - [`ticket`] — kartu tiket (komposisi countdown + papan jadwal).
//! - [`countdown`] — pane kiri tiket (identitas + hitung mundur + tanggal).
//! - [`prayer_table`] — papan jadwal hari ini + lonceng adzan per baris.
//! - [`ui`] — komponen atomik (tombol, modal, sakelar, …).

pub mod countdown;
pub mod prayer_table;
pub mod ticket;
pub mod title_bar;
pub mod ui;
pub mod widget;
