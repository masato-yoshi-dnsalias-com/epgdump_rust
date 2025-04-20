extern crate getopts;

use chrono::{DateTime, Local, TimeZone};
use chrono::prelude::{Datelike, Timelike};
use colored::*;
use std::io::{BufRead};
use env_logger::{Builder, Env, Target};
use std::fs::File;
use std::path::Path;
use getopts::Options;
use log::{debug, warn};
use std::env;
//use std::io::prelude::*;
use std::io::{BufReader, Write};
use std::process;

mod arib;
mod eit;
mod sdt;
mod ts;

use crate::eit::{CERTAINTY, START_TIME_UNCERTAINTY, DURATION_UNCERTAINTY};
use crate::eit::{dump_eit};
use crate::sdt::{dump_sdt};
use crate::ts::{MAXSECBUF, read_ts, SecCache, SvtControl, SvtControlTop,
                EitControl, TsPacket, TSPAYLOADMAX};

// 定数設定
pub const PROGRAM:  &str = env!("CARGO_PKG_NAME");   // パッケージ名
pub const VERSION: &str = env!("CARGO_PKG_VERSION"); // パッケージバージョン

const CAP: usize = 188 * 1;
const SECCOUNT: usize = 64;


// Usage出力
fn show_usage(program: &str, opts: &Options) {

    let brief = format!("Usage: {} --BS|--CS|<id> tsFile outfile [ ( [--pf] [--sid n] ) | [--cut n1,n2] ]", program);
    eprintln!("{}", opts.usage(&brief));

}

// コマンドラインオプション構造体
#[derive(Debug, Clone)]
struct CommanLineOpt {
    //is_logo: bool,
    sdt_mode: bool,
    eit_mode: bool,
    is_xml: bool,
    is_sid: bool,
    select_sid: i32,
    _is_cut: bool,
    cut_sid_list: String,
    is_bs: bool,
    is_cs: bool,
    //is_time: bool,
    id: String,
    infile: String,
    outfile: String,
}

pub(crate) fn command_line_check(program: &str) -> CommanLineOpt {

    //let mut _is_logo: bool = false;
    let mut sdt_mode: bool = false;
    let mut eit_mode: bool = false;
    let mut is_xml: bool = false;
    let mut is_sid: bool = false;
    let mut select_sid: i32 = 0;
    let mut is_cut: bool = false;
    let mut cut_sid_list: String = "".to_string();
    let mut is_bs: bool = false;
    let mut is_cs: bool = false;
    //let mut _is_time: bool = false;
    let mut id: String = "".to_string();
    let infile: String;
    let outfile: String;


    // 実行時に与えられた引数をargs: Vec<String>に格納する
    let mut args: Vec<String> = env::args().collect();

    // epgdump互換引数の変換
    for cnt in 0..args.len() {

        let change_string = match &*args[cnt] {
            "/BS"  => { String::from("--BS") },
            "/CS"  => { String::from("--CS") },
            "-all" => { String::from("--all") },
            "-cut" => { String::from("--cut") },
            "-pf"  => { String::from("--pf") },
            "-sid" => { String::from("--sid") },
            "-xml" => { String::from("--xml") },
            _      => { String::from("") },
        };

        // args: Vec<String>の更新
        if change_string != "" {
            args[cnt] = change_string;
        }
    }

    // オプションを設定
    let mut opts = Options::new();
    //opts.optflag("","LOGO","ロゴ取得モード。独立して指定し、番組表の出力を行ないません。\n必要なTSの長さ 地上波は10分 BS/CSは20分です。");
    opts.optflag("","BS","/BS,BSモード。一つのTSからBS全局のデータを読み込みます。");
    opts.optflag("","CS","/CS,CSモード。一つのTSからCS複数局のデータを読み込みます。");
    //opts.optopt("","","チャンネル識別子。地上波の物理チャンネルを与えます。","id");
    //opts.optopt("","TIME","時刻合わせモード。TSからTOT(TimeOffsetTable)を読み込みます。\nrecpt1 <任意> 10(秒以上) - | epgdump --TIME - <任意> の形で使用してください。\nTOTは5秒に1回しか来ないため、recpt1に与える時間をある程度長くしてください。","");
    opts.optflag("","pf","-pf,EID[pf]単独出力モード。必要なTSの長さは4秒です。");
    opts.optopt("","sid","-sid,BS/CS単チャンネル出力モード。nにはチャンネルsidを指定","n");
    opts.optopt("c","cut","-cut,BS/CS不要チャンネル除外モード。nには不要チャンネルsidをcsv形式で指定","n1,n2,...");
    opts.optflag("","all","-all,全サービスを出力対象とする。");
    opts.optflag("","xml","-xml,XMLフォーマットで出力する。");
    opts.optflag("h","help","このヘルプを表示");
    opts.optflag("v","version","バージョンを表示する。");

    // 未定義のオプションを指定した場合にエラーメッセージを出力する
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(msg) => {
            eprintln!("Error: {}", msg.to_string());
            show_usage(&program, &opts);
            process::exit(0);
        }
    };

    // ヘルプを表示し終了
    if matches.opt_present("help") {
        show_usage(&program, &opts);
        process::exit(0);
    }

    // バージョンを表示し終了
    if matches.opt_present("version") {
        eprintln!("{} {}",program, VERSION);
        process::exit(0);
    }

    // LOGOモードの設定
    //if matches.opt_present("LOGO") {
    //    _is_logo = true;
    //    is_xml = true;
    //}

    // BSオプションの設定
    if matches.opt_present("BS") {
      is_bs = true;
    }

    // CSオプションの設定
    if matches.opt_present("CS") {
        is_cs = true;
    }

    // TIMEオプションの設定
    //if matches.opt_present("TIME") {
    //    is_time = true;
    //}

    // EID[pf]単独出力モードの設定
    if matches.opt_present("pf") {
        eit_mode = true;
    }

    // 全サービス出力の設定
    if matches.opt_present("all") {
        sdt_mode = true;
    }

    // XMLフォーマット出力の設定
    if matches.opt_present("xml") {
        is_xml = true;
    }

    // BS/CS単チャンネル出力モードの設定
    if matches.opt_present("sid") {
        is_sid = true;
        select_sid = match matches.opt_str("sid").unwrap().parse::<i32>() {
            Ok(select_sid) => select_sid,
            Err(_e) => {
                show_usage(&program, &mut &opts);
                process::exit(0);
            },
        };
    }

    // BS/CS不要チャンネル除外モードの設定
    if matches.opt_present("cut") {
        is_cut = true;
        cut_sid_list = matches.opt_str("cut").unwrap().to_string();
    }
    
    // 引数（オプションを除く）判定処理
    match matches.free.len() {
        2 if is_bs == true || is_cs == true => {

            infile = matches.free[0].clone();
            outfile = matches.free[1].clone();

        },
        3 if (is_bs == false && is_cs == false) => {

            if matches.free[0].to_uppercase().starts_with("GR") {

                id = matches.free[0].clone();
                infile = matches.free[1].clone();
                outfile = matches.free[2].clone();

            }
            else {

                show_usage(&program, &mut &opts);
                process::exit(0);

            };

        },
        _ => {
            show_usage(&program, &mut &opts);
            process::exit(0);
        },

    };

    // リターン情報
    CommanLineOpt {
        //is_logo: _is_logo,
        sdt_mode: sdt_mode,
        eit_mode: eit_mode,
        is_xml: is_xml,
        is_sid: is_sid,
        select_sid: select_sid,
        _is_cut: is_cut,
        cut_sid_list: cut_sid_list,
        is_bs: is_bs,
        is_cs: is_cs,
        //is_time: _is_time,
        id: id,
        infile: infile,
        outfile: outfile,
    }

}

struct TsidList {
    tsid: u32,
    node: u32,
    slot: u32,
}

//
// Tsidリスト読込み処理
//
fn tsid_node_slot_list_read(tsid_list: &mut Vec<TsidList>) {

const LIST_FILE: [&str; 2] = [
    "/etc/epgdump/tsid.conf",
    "/usr/local/etc/epgdump/tsid.conf",
];

    // 定義ファイルループ
    for cnt in 0..LIST_FILE.len() {

        // 定義ファイルの存在チェック
        if Path::new(LIST_FILE[cnt]).exists() {
            debug!("tsid list file = {}",LIST_FILE[cnt]);

            // ファイルオープン処理
            let file = match File::open(LIST_FILE[cnt].to_string()) {
                Ok(file) => file,
                Err(err) => {

                    // オープンエラーの場合はログシュル得して継続
                    warn!("File Open Error({})", err);

                    continue

                },
            };

            // 入力バッファーの作成
            let buffer = BufReader::new(file);

            // ファイルからリード処理(１行づつ)
            for line in buffer.lines() {

                // １行データのtrim処理
                let line_data = line.unwrap().trim().to_string();

                // 行頭が「#」以外取り込み（「#」はコメント行）
                if line_data.chars().nth(0) != Some('#') {

                    // 入力データを「,」で分割
                    let tsid_data: Vec<&str> = line_data.split(',').collect();

                    // データが３つともある場合に構造体に作成
                    if tsid_data[0] != "" && tsid_data[1] != "" && tsid_data[2] != "" {
                        debug!("tsid={},node={},slot={}", tsid_data[0], tsid_data[1], tsid_data[2]);

                        // 構造体データ作成
                        tsid_list.push(TsidList {
                            tsid: tsid_data[0].parse().unwrap(),
                            node: tsid_data[1].parse().unwrap(),
                            slot: tsid_data[2].parse().unwrap(),
                        });

                    };
                };
            };
            
            // ループ終了
            break;

        };
    };

}

fn main() {

    // env_logの初期化
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let local_time = Local::now().format("%Y/%m/%d %H:%M:%S");
            let level = match record.level() {
                log::Level::Error => "ERROR  ".red(),
                log::Level::Warn  => "WARNING".yellow(),
                log::Level::Info  => "INFO   ".green(),
                log::Level::Debug => "DEBUG  ".cyan(),
                log::Level::Trace => "TRACE  ".blue(),
            };
            let pid = process::id();
            writeln!(
                buf,
                "[{}] {} {} [{}] {}",
                local_time,
                level,
                record.target(),
                pid,
                record.args(),
            )
        }
    )
    .target(Target::Stdout)  // 出力先をStdoutに変更
    .init();

    // コマンドラインチェック処理
    let opt = command_line_check(PROGRAM);

    // チャンネルタイプの設定
    let ch_type;
    if opt.is_bs == true {
        ch_type = 1;
    }
    else if opt.is_cs == true {
        ch_type = 2;
    }
    else {
        ch_type = 0;
    };

    // tsid_list構造体の作成と初期化
    let mut tsid_list: Vec<TsidList> = vec![];

    // tsid_listデータの読み込み
    tsid_node_slot_list_read(&mut tsid_list);

    // 構造体(SecCache, TS_Packet)の初期化
    let mut secs: [SecCache; SECCOUNT] = [
        SecCache {
            pid: 0,
            buf: [0xff; MAXSECBUF + 1],
            seclen: 0,
            setlen: 0,
            cur: TsPacket {
                sync: 0,
                transport_error_indicator: 0,
                payload_unit_start_indicator: 0,
                transport_priority: 0,
                pid: 0,
                transport_scrambling_control: 0,
                adaptation_field_control: 0,
                continuity_counter: 0,
                adaptation_field: 0,
                payload: [0; TSPAYLOADMAX],
                payloadlen: 0,
                rcount: 0,
            },
            curlen: 0,
            cont: 0,
        }; SECCOUNT
    ];

    // svttop構造体の作成と初期化
    let mut svttop: Vec<SvtControlTop> = vec![];
    svttop.push(SvtControlTop {
        service_id: 0,
        svt_control_sub: vec![],
    });
   
    // SvtControlTopの初期化
    svttop[0].svt_control_sub.push(SvtControl {
        service_id: 0,
        service_type: 0x00,
        original_network_id: 0,
        transport_stream_id: 0,
        slot: 0,
        //servicename: [0; MAXSECLEN],
        servicename: String::new(),
        ontv: String::new(),
        eitsch: vec![],
        eit_pf: vec![],
        prev_sch: vec![],
        import_cnt: 0,
        import_stat: 0,
        logo_download_data_id: 0,
        logo_version: 0,
    });

    // SIDを指定している場合にSvtControlTopに格納エリアの追加とデータの初期化
    if opt.is_sid == true {
        svttop.push(SvtControlTop{
            service_id: opt.select_sid,
           svt_control_sub: vec![],
        });
        let svttop_len = svttop.len();
        svttop[svttop_len - 1].svt_control_sub.push(SvtControl {
            service_id: 0,
            service_type: 0x00,
            original_network_id: 0,
            transport_stream_id: 0,
            slot: 0,
            servicename: String::new(),
            ontv: String::new(),
            eitsch: vec![],
            eit_pf: vec![],
            prev_sch: vec![],
            import_cnt: 0,
            import_stat: 0,
            logo_download_data_id: 0,
            logo_version: 0,
        });
    };

    // インプットTSファイルのオープン
    let infile = match &*opt.infile {
        "-" => {
            File::open("/dev/stdin").unwrap()
        },
        _ => {
            File::open(&opt.infile).unwrap()
        },
    };

    // アウトプットファイルのオープン
    let mut outfile = match &*opt.outfile {
        "-" => {
            File::create("/dev/stdout").unwrap()
        },
        _ => {
            File::create(&opt.outfile).unwrap()
        },
    };

    // リードバッファの作成
    let mut readbuff_file = BufReader::with_capacity(CAP, &infile);

    let secs_count: usize;

    // 処理対象を設定
    secs[1].pid = 0x11; // SDT
    secs[2].pid = 0x12; // H-EIT
    secs_count = 3;

    // std取得呼び出し
    get_sdt(&opt, &mut readbuff_file, &mut svttop, &mut secs, secs_count);
    
    // 不要なsevice_idの削除
    for cnt in (0..svttop.len()).rev() {

        if svttop[cnt].svt_control_sub[0].import_stat <= 0 {

            svttop.remove(cnt);

        };

    };

    // 削除対象とするtransport_stream_idのワーク変数
    let mut transport_stream_id = 0;

    // 余計なeit_pfの削除
    for cnt in 0..svttop.len() {

        // 2つ目以降のeit_pfを削除
        if transport_stream_id != svttop[cnt].svt_control_sub[0].transport_stream_id {

            // 削除対象のtransport_stream_idを設定
            transport_stream_id = svttop[cnt].svt_control_sub[0].transport_stream_id;

        }
        else {

            // eit_pfの削除
            svttop[cnt].svt_control_sub[0].eit_pf = vec![];

        };

        debug!("svttop={},service_id={},service_type={},servicename={},import_cnt={},import_stat={}", cnt, svttop[cnt].svt_control_sub[0].service_id, svttop[cnt].svt_control_sub[0].service_type, svttop[cnt].svt_control_sub[0].servicename, svttop[cnt].svt_control_sub[0].import_cnt, svttop[cnt].svt_control_sub[0].import_stat);

    };

    // XMLファイルの作成処理
    if opt.is_xml == true {

        // ヘッダー出力
        writeln!(outfile,"<?xml version=\"1.0\" encoding=\"UTF-8\"?>").unwrap();
        writeln!(outfile,"<!DOCTYPE tv SYSTEM \"xmltv.dtd\">\n").unwrap();
        writeln!(outfile,"<tv generator-info-name=\"tsEPG2xml\" generator-info-url=\"http://localhost/\">").unwrap();
        
        // svttop配列分ループ
        for cnt in 0..svttop.len() {

            // サービス名変数の作成
            let service_name = xml::escape::escape_str_attribute(&svttop[cnt].svt_control_sub[0].servicename);

            // サブヘッダー出力
            writeln!(outfile,"  <channel id=\"{}\">", &svttop[cnt].svt_control_sub[0].ontv).unwrap();
            writeln!(outfile,"    <display-name lang=\"ja_JP\">{}</display-name>", service_name).unwrap();
            writeln!(outfile,"    <id ts=\"{}\" on=\"{}\" sv=\"{}\" st=\"{}\"/>", 
                &svttop[cnt].svt_control_sub[0].transport_stream_id, &svttop[cnt].svt_control_sub[0].original_network_id,
                &svttop[cnt].svt_control_sub[0].service_id, &svttop[cnt].svt_control_sub[0].service_type).unwrap();
            writeln!(outfile,"  </channel>").unwrap();

        };

        // svttop配列分ループ
        for cnt in 0..svttop.len() {

            // XML詳細作成処理呼び出し
            dump_xml(&opt, &mut outfile, &mut svttop[cnt].svt_control_sub[0]);

        };

        // フッター出力
        writeln!(outfile,"</tv>").unwrap();

    }
    // svttop配列がある場合のserial出力処理
    else if svttop.len() > 0 {

        // ヘッダー出力
        write!(outfile, "a:{}:{}", svttop.len(), "{".to_string()).unwrap();
        let mut sdt_cnt = 0;

        // svttop配列分ループ
        for cnt in 0..svttop.len() {

            // 変数初期化
            let mut node = 0;
            let mut slot = 0;

            // チャンネルタイプによるデータ作成
            // BS/CS
            if ch_type != 0 {

                // TSIDからNodeとSlotを生成処理
                node = (svttop[cnt].svt_control_sub[0].transport_stream_id & 0x1f0) >> 4;
                slot = svttop[cnt].svt_control_sub[0].transport_stream_id & 0x07;

                // TSIDからNodeとSlotをテーブルから変換処理
                for cnt2 in 0..tsid_list.len() {

                    // TsidListに対象のTSIDがある場合に変換
                    if svttop[cnt].svt_control_sub[0].transport_stream_id == tsid_list[cnt2].tsid {

                        // TsidListから設定
                        node = tsid_list[cnt2].node;
                        slot = tsid_list[cnt2].slot;
                        debug!("BS Channel Node Slot Change(Tsid={},Node={},Slot={})", 
                            tsid_list[cnt2].tsid, tsid_list[cnt2].node, tsid_list[cnt2].slot);

                        // ループ終了
                        break

                    };
                };
            };

            // 地上波
            if ch_type == 1 && 
                (svttop[cnt].svt_control_sub[0].transport_stream_id == 0x40f1 || 
                 svttop[cnt].svt_control_sub[0].transport_stream_id == 0x40f2) {
                slot -= 1;
            };

            // サブヘッダー出力
            write!(outfile, "i:{};a:8:{}", sdt_cnt, "{").unwrap();
            write!(outfile, "s:2:\"id\";s:{}:\"{}\";",
                svttop[cnt].svt_control_sub[0].ontv.len(), &svttop[cnt].svt_control_sub[0].ontv).unwrap();
            write!(outfile, "s:12:\"display-name\";s:{}:\"{}\";",
                svttop[cnt].svt_control_sub[0].servicename.len(), &svttop[cnt].svt_control_sub[0].servicename).unwrap();
            write!(outfile, "s:2:\"ts\";i:{};", &svttop[cnt].svt_control_sub[0].transport_stream_id).unwrap();
            write!(outfile, "s:2:\"on\";i:{};", &svttop[cnt].svt_control_sub[0].original_network_id).unwrap();
            write!(outfile, "s:2:\"sv\";i:{};", &svttop[cnt].svt_control_sub[0].service_id).unwrap();
            write!(outfile, "s:2:\"st\";i:{};", &svttop[cnt].svt_control_sub[0].service_type).unwrap();
            write!(outfile, "s:4:\"node\";i:{};", node).unwrap();
            write!(outfile, "s:4:\"slot\";i:{};{}", slot, "}").unwrap();

            sdt_cnt += 1;

        };
        writeln!(outfile, "{}", "}").unwrap();

        // 詳細出力
        for cnt in 0..svttop.len() {

            if svttop[cnt].svt_control_sub[0].import_stat == 2 {

                // シリアル出力処理呼び出し
                dump_serial(&opt, &mut outfile, &mut svttop[cnt].svt_control_sub[0])

            };

        };
    };

}

//
// データ構造体作成処理
//
fn get_sdt( cmd_opt: &CommanLineOpt, mut readbuff_file: &mut BufReader<&File>,
    svttop: &mut Vec<SvtControlTop>, mut secs: &mut [SecCache], count: usize) -> () {

    // ループ
    loop {

        // ファイルリード
        let bsecs = read_ts(&mut readbuff_file, &mut secs, count);

        // リードデータ有無判定
        match bsecs {
            Some(bsecs) => {  // リードデータ有り

                // PID判定処理
                match bsecs.pid & 0xff {
                    0x11 => {  // SDT

                        // SDT構造体の作成処理呼び出し
                        dump_sdt(&cmd_opt, &bsecs.buf, svttop); 

                    },
                    0x12 => {  // EIT

                        // EIT構造体の作成処理呼び出し
                        dump_eit(&cmd_opt, &bsecs.buf, svttop);

                    },
                    /*
                    0x14 => {  // TOT

                    },
                    // SDTT
                    0x24 => {  // SDTT
                     
                    },
                    */
                    _ => {  // デフォルト(無処理)

                    },
                };
            },
            None => {  // リードデータ無し
                       //
                debug!("bsecs None");
                break;

            },
        };
                
    };
}

//
// 放送休止データ挿入処理(EIT PF)
//
fn insert_rest_pf(svtcur: &mut SvtControl) -> () {

    // 変数作成
    let mut start_time: i64;
    let mut end_time: i64;
    let mut end_time_dt: DateTime<Local>;

    // eit_pf配列が未作成の場合はリターン
    if svtcur.eit_pf.len() <= 0 {

        return;

    }

    // イベントステータスがCERTAINTYの場合の処理
    if svtcur.eit_pf[0].event_status == CERTAINTY {

        // end_time用のDateTime形式の開始日時情報作成
        let dt: DateTime<Local> = Local.with_ymd_and_hms(
            svtcur.eit_pf[0].yy as i32 + 1900, svtcur.eit_pf[0].mm as u32, svtcur.eit_pf[0].dd as u32,
            svtcur.eit_pf[0].hh as u32, svtcur.eit_pf[0].hm as u32, svtcur.eit_pf[0].ss as u32).unwrap();

        // end_time用のシリアル形式の終了日時情報作成
        end_time = dt.timestamp() + svtcur.eit_pf[0].duration as i64;

        // end_time用のDateTime形式の終了日時情報作成
        end_time_dt = Local.timestamp_opt(end_time,0).unwrap();

        // eit_pfループカウンター作成
        let mut cnt2 = 1;

        // eit_pf配列分ループ
        while cnt2 < svtcur.eit_pf.len() {

            // イベントステータスがSTART_TIME_UNCERTAINTYより大きい場合はリターン
            if (svtcur.eit_pf[cnt2].event_status & START_TIME_UNCERTAINTY) > 0 {

                return;

            }

            // starttime用のDateTime形式の開始日時情報作成
            let dt: DateTime<Local> = Local.with_ymd_and_hms(
                svtcur.eit_pf[cnt2].yy as i32 + 1900, svtcur.eit_pf[cnt2].mm as u32, svtcur.eit_pf[cnt2].dd as u32,
                svtcur.eit_pf[cnt2].hh as u32, svtcur.eit_pf[cnt2].hm as u32, svtcur.eit_pf[cnt2].ss as u32).unwrap();

            // start_time用のシリアル形式の開始日時情報作成
            start_time = dt.timestamp();

            // 終了時間が開始時間より小さい場合は放送休止データを挿入
            if end_time != start_time {
                svtcur.eit_pf.insert(cnt2, EitControl {
                    table_id: svtcur.eit_pf[cnt2].table_id,
                    servid: svtcur.eit_pf[cnt2].servid,
                    event_id: -1,
                    version_number: 0,
                    section_number: 0,
                    last_section_number: 0,
                    segment_last_section_number: 0,
                    running_status: 0,
                    free_ca_mode: 0,
                    content_type: 14,
                    content_subtype: 0x3f,
                    genre2: 16,
                    sub_genre2: 16,
                    genre3: 16,
                    sub_genre3: 16,
                    episode_number: 0,
                    yy: end_time_dt.year() as i32 - 1900,
                    mm: end_time_dt.month() as i32,
                    dd: end_time_dt.day() as i32,
                    hh: end_time_dt.hour() as i32,
                    hm: end_time_dt.minute() as i32,
                    ss: end_time_dt.second() as i32,
                    duration: (start_time - end_time) as i32,
                    start_time: start_time + svtcur.eit_pf[cnt2 - 1].duration as i64,
                    title: String::from("放送休止"),
                    subtitle: String::new(),
                    desc: String::new(),
                    desc_length: 0,
                    video_type: 0,
                    audio_type: 0,
                    multi_type: 0,
                    event_status: 0,
                    sch_pnt: -1,
                    import_cnt: 0,
                    renew_cnt: 0,
                    tid: 0,
                    tid_status: 0,
                });

                // 放送休止追加後にカウンターアップ
                cnt2 += 1;

            };

            // イベントステータスがDURATION_UNCERTAINTYより大きい場合はリターン
            if (svtcur.eit_pf[cnt2].event_status & DURATION_UNCERTAINTY) > 0 {

                return;

            }

            // 終了日時を更新
            end_time = start_time + svtcur.eit_pf[cnt2].duration as i64;

            // DateTime形式の終了日時情報作成
            end_time_dt = Local.timestamp_opt(end_time,0).unwrap();

            // カウンターアップ
            cnt2 += 1;

        }
    }
}

//
// 放送休止データ挿入処理(EIT SCH)
//
fn insert_rest_sch(svtcur: &mut SvtControl) -> () {

    // 変数作成
    let mut start_time: i64;
    let mut end_time: i64;
    let mut end_time_dt: DateTime<Local>;
    let cnt = 0;

    // DateTime形式の開始日時情報作成
    let dt: DateTime<Local> = Local.with_ymd_and_hms(
        svtcur.eitsch[cnt].yy as i32 + 1900, svtcur.eitsch[cnt].mm as u32, svtcur.eitsch[cnt].dd as u32,
        svtcur.eitsch[cnt].hh as u32, svtcur.eitsch[cnt].hm as u32, svtcur.eitsch[cnt].ss as u32).unwrap();

    // シリアル形式の終了日時情報作成
    end_time = dt.timestamp() + svtcur.eitsch[cnt].duration as i64;

    // DateTime形式の終了日時情報作成
    end_time_dt = Local.timestamp_opt(end_time,0).unwrap();

    // eitschループカウンター作成
    let mut cnt2 = 0;

    // eitsch配列分ループ
    while cnt2 < svtcur.eitsch.len() {

        // DateTime形式の開始日時情報作成
        let dt: DateTime<Local> = Local.with_ymd_and_hms(
            svtcur.eitsch[cnt2].yy as i32 + 1900, svtcur.eitsch[cnt2].mm as u32, svtcur.eitsch[cnt2].dd as u32,
            svtcur.eitsch[cnt2].hh as u32, svtcur.eitsch[cnt2].hm as u32, svtcur.eitsch[cnt2].ss as u32).unwrap();

        // シリアル形式の開始日時情報作成
        start_time = dt.timestamp();
            
        // 終了時間が開始時間より小さい場合は放送休止データを挿入
        if end_time < start_time {

            svtcur.eitsch.insert(cnt2, EitControl {
                table_id: svtcur.eitsch[cnt2].table_id,
                servid: svtcur.eitsch[cnt2].servid,
                event_id: -1,
                version_number: 0,
                section_number: 0,
                last_section_number: 0,
                segment_last_section_number: 0,
                running_status: 0,
                free_ca_mode: 0,
                content_type: 14,
                content_subtype: 0x3f,
                genre2: 16,
                sub_genre2: 16,
                genre3: 16,
                sub_genre3: 16,
                episode_number: 0,
                yy: end_time_dt.year() as i32 - 1900,
                mm: end_time_dt.month() as i32,
                dd: end_time_dt.day() as i32,
                hh: end_time_dt.hour() as i32,
                hm: end_time_dt.minute() as i32,
                ss: end_time_dt.second() as i32,
                duration: (start_time - end_time) as i32,
                start_time: end_time as i64,
                title: String::from("放送休止"),
                subtitle: String::new(),
                desc: String::new(),
                desc_length: 0,
                video_type: 0,
                audio_type: 0,
                multi_type: 0,
                event_status: 0,
                sch_pnt: 0,
                import_cnt: 0,
                renew_cnt: 0,
                tid: 0,
                tid_status: 0,
            });

            // 放送休止追加後にカウンターアップ
            cnt2 += 1;

        };

        // 終了日時を更新
        end_time = start_time + svtcur.eitsch[cnt2].duration as i64;

        // DateTime形式の終了日時情報作成
        end_time_dt = Local.timestamp_opt(end_time,0).unwrap();

        // カウンターアップ
        cnt2 += 1;
    }
}

//
// ジャンル未定義補正処理
//
fn rest_repair(eitcur: &mut EitControl) -> () {

    if eitcur.content_type == 0 && eitcur.content_subtype == 0 && eitcur.genre2 == 0 &&
        eitcur.sub_genre2 == 0 && eitcur.genre3 == 0 && eitcur.sub_genre3 == 0 {

        eitcur.content_type    = 14;
        eitcur.content_subtype = 0x3f;
        eitcur.genre2          = 16;
        eitcur.sub_genre2      = 16;
        eitcur.genre3          = 16;
        eitcur.sub_genre3      = 16;

    }

    if eitcur.content_type == 14 && eitcur.content_subtype == 0x3f && eitcur.title == "" {

        eitcur.title = "放送休止".to_string();

    }

}

//
// シリアルデータの作成処理
//
fn line_serial(line_cnt: i32, array_cnt: i32, mut eitcur: &mut EitControl, ch_disc: &String) -> String {

    // ジャンル未定義補正処理呼び出し
    rest_repair(&mut eitcur);
    
    // 処理用タイトル作成
    let title = &eitcur.title;

    // 処理用サブタイトル作成
    let mut subtitle = String::new();
    if eitcur.free_ca_mode > 1 {
        subtitle = "[￥]".to_string();
    }
    subtitle.push_str(&eitcur.subtitle);

    // 処理用開始時間作成
    let start_time = eitcur.start_time;
    
    // 処理用終了時間作成
    let end_time = start_time + eitcur.duration as i64;

    // 処理用終了時間(キャラクタ)作成
    let cendtime = Local.timestamp_opt(end_time,0).unwrap().format("%Y-%m-%d %H:%M:%S");

    // 処理用開始時間(キャラクタ)作成
    let cstarttime = format!("{:4}-{:02}-{:02} {:02}:{:02}:{:02}", eitcur.yy + 1900, eitcur.mm, eitcur.dd, eitcur.hh, eitcur.hm, eitcur.ss);

    // 処理用コンテンツタイプ作成
    let content_type = match eitcur.content_type {
        16 => { 0 },
        _ => { eitcur.content_type + 1 },
    };

    // 処理用genre2作成
    let genre2 = match eitcur.genre2 {
        16 => { 0 },
        _ => { eitcur.genre2 + 1 },
    };

    // 処理用genrer3作成
    let genre3 = match eitcur.genre3 {
        16 => { 0 },
        _ => { eitcur.genre3 + 1 },
    };
    
    // リターン文字作成
    let ret_str = format!(
        "i:{};a:{}:{}\
        s:9:\"starttime\";s:19:\"{}\";\
        s:7:\"endtime\";s:19:\"{}\";\
        s:12:\"channel_disc\";s:{}:\"{}\";\
        s:3:\"eid\";i:{};\
        s:5:\"title\";s:{}:\"{}\";\
        s:4:\"desc\";s:{}:\"{}\";\
        s:8:\"category\";i:{};s:9:\"sub_genre\";i:{};\
        s:6:\"genre2\";i:{};s:10:\"sub_genre2\";i:{};\
        s:6:\"genre3\";i:{};s:10:\"sub_genre3\";i:{};\
        s:10:\"video_type\";i:{};s:10:\"audio_type\";i:{};s:10:\"multi_type\";i:{};",
        line_cnt,array_cnt,"{",
        cstarttime,
        cendtime,
        ch_disc.len(),ch_disc,
        eitcur.event_id,
        title.len(),title,
        subtitle.len(), subtitle,
        content_type, eitcur.content_subtype,
        genre2, eitcur.sub_genre2,
        genre3, eitcur.sub_genre3,
        eitcur.video_type, eitcur.audio_type, eitcur.multi_type
        );

    // リターン情報
    ret_str.to_string()

}

//
// sch_pntの補正処理
//
fn sch_pnt_update( svtcur: &mut SvtControl) -> () {

    // カウンター作成
    let mut pf_cnt = 0;
    let mut sch_cnt = 0;

    // eit_pfの配列分ループ
    for cnt in 0..svtcur.eit_pf.len() {

        // eitschの配列分ループ
        for cnt2 in sch_cnt as usize..svtcur.eitsch.len() {

            // eit_pfとeitschで同じイベントIDを見つけた場合の処理
            if svtcur.eit_pf[cnt].event_id == svtcur.eitsch[cnt2].event_id {

                // 最初のeit_pfでかつeitschが先頭より大きい場合
                if pf_cnt == 0 && sch_cnt > 1 {

                    // 冒頭の余分なschをpf1つ前を残して切り捨て
                    for _cnt3 in 0..cnt2 - 1 {

                        svtcur.eitsch.remove(0);

                    }

                    // sch_cntカウンター変更
                    sch_cnt = 1;
                };

                // sch_pntにsch_cntを設定
                svtcur.eit_pf[cnt].sch_pnt = sch_cnt as i32;

                // 次を処理
                break;

            }
            else {

                // ループ終了時に同じイベントＩＤが見つからなかった場合の処理
                if cnt2 == svtcur.eitsch.len() -1 {

                    // 未発見フラグ設定
                    svtcur.eit_pf[cnt].sch_pnt = -1;

                };

            };

            // schカウントアップ
            sch_cnt += 1;

            // ループ終了時にカウンタークリア
            if cnt2 == svtcur.eitsch.len() -1 {

                sch_cnt = 0;

            };
        }

        // pfカウントアップ
        pf_cnt += 1;

    }

}

//
// シリアル出力処理
//
fn dump_serial( cmd_opt: &CommanLineOpt, outfile: &mut File, mut svtcur: &mut SvtControl) -> () {

    // 放送休止補正処理(EIT PF)
    insert_rest_pf(&mut svtcur);

    if cmd_opt.eit_mode == false && svtcur.eitsch.len() > 0 {

        // 放送休止補正処理(EIT SCH)
        insert_rest_sch(&mut svtcur);

        // sch_pnt補正処理呼出し
        sch_pnt_update(&mut svtcur);

        // eit_pf、eitschにデータがある場合に出力処理
        if svtcur.eit_pf.len() > 0 || svtcur.eitsch.len() > 0 {

            writeln!(outfile,"a:3:{}s:4:\"disc\";s:{}:\"{}\";s:6:\"pf_cnt\";i:{};s:7:\"sch_cnt\";i:{};{}",
                "{", svtcur.ontv.len(), svtcur.ontv, svtcur.eit_pf.len(), svtcur.eitsch.len(), "}").unwrap();

        }
        // ない場合はリターン
        else {

            return;

        };

        // eit_pfにデータがある場合に出力処理
        if  svtcur.eit_pf.len() > 0 {

            write!(outfile,"a:{}:{}", svtcur.eit_pf.len(), "{").unwrap();

            // eit_pf配列分ループし出力
            for cnt in 0..svtcur.eit_pf.len() {

                write!(outfile, "{}",
                    line_serial(cnt as i32, 17, &mut svtcur.eit_pf[cnt], &svtcur.ontv)).unwrap();
                write!(outfile, "s:6:\"status\";i:{};s:7:\"sch_pnt\";i:{};{}",
                    svtcur.eit_pf[cnt].event_status, svtcur.eit_pf[cnt].sch_pnt, "}").unwrap();

            }

            writeln!(outfile, "{}", "}").unwrap();

        }

        // eitschにデータがある場合に出力処理
        if svtcur.eitsch.len() > 0 {

            write!(outfile,"a:{}:{}", svtcur.eitsch.len(), "{").unwrap();

            // eitsch配列分ループし出力
            for cnt in 0..svtcur.eitsch.len() {

                write!(outfile, "{}{}",
                    line_serial(cnt as i32, 15, &mut svtcur.eitsch[cnt], &svtcur.ontv), "}").unwrap();

            }

            writeln!(outfile, "{}", "}").unwrap();

        }
    }
}

//
// xmlキャラクタ変換処理
//
fn xml_special_chars(xml: String) -> String {

    let mut ret_string = xml.clone();

    ret_string = ret_string.replace("&", "&amp;");
    ret_string = ret_string.replace("'", "&apos;");
    ret_string = ret_string.replace("\"", "&quot;");
    ret_string = ret_string.replace("<", "&lt;");
    ret_string = ret_string.replace(">", "&gt;");
    
    // リターン情報
    ret_string
}

//
// xmlデータ作成処理
//
fn dump_xml( cmd_opt: &CommanLineOpt, outfile: &mut File, mut svtcur: &mut SvtControl) -> () {

    // 放送休止補正処理(EIT PF)
    insert_rest_pf(&mut svtcur);

    // EITモードフラグがfalseでeitschにデータある場合の処理
    if cmd_opt.eit_mode == false && svtcur.eitsch.len() > 0 {

        // 放送休止補正処理(EIT SCH)
        insert_rest_sch(&mut svtcur);

        // sch_pnt補正処理呼出し
        sch_pnt_update(&mut svtcur);

        // eit_pfにデータがある場合の処理
        if svtcur.eit_pf.len() > 0 {

            // eit_pf配列分ループ
            for cnt in 0..svtcur.eit_pf.len() {

                // ジャンル未定義補正処理呼び出し
                rest_repair(&mut svtcur.eit_pf[cnt]);

                // 処理用タイトル作成
                let mut title = svtcur.eit_pf[cnt].title.clone();
                title = xml_special_chars(title);

                // 処理用サブタイトル作成
                let mut subtitle = svtcur.eit_pf[cnt].subtitle.clone();
                subtitle = xml_special_chars(subtitle);

                // 処理用タグ作成
                let tag = "programme_pf".to_string();

                // 処理用開始日時作成
                let start_time = svtcur.eit_pf[cnt].start_time;

                // 処理用終了日時作成
                let end_time = start_time + svtcur.eit_pf[cnt].duration as i64;

                // 処理用終了日時(キャラクタ)作成
                let cendtime = Local.timestamp_opt(end_time,0).unwrap().format("%Y-%m-%d %H:%M:%S");

                // 処理用開始日時(キャラクタ)作成
                let cstarttime = format!("{:4}-{:02}-{:02} {:02}:{:02}:{:02}",
                    svtcur.eit_pf[cnt].yy + 1900, svtcur.eit_pf[cnt].mm, svtcur.eit_pf[cnt].dd,
                    svtcur.eit_pf[cnt].hh, svtcur.eit_pf[cnt].hm, svtcur.eit_pf[cnt].ss);

                // 処理用コンテンツタイプ作成
                let content_type = match svtcur.eit_pf[cnt].content_type {
                    16 => { 0 },
                    _ => { svtcur.eit_pf[cnt].content_type + 1 },
                };

                // 処理用genre2作成
                let genre2 = match svtcur.eit_pf[cnt].genre2 {
                    16 => { 0 },
                    _ => { svtcur.eit_pf[cnt].genre2 + 1 },
                };

                // 処理用genre3作成
                let genre3 = match svtcur.eit_pf[cnt].genre3 {
                    16 => { 0 },
                    _ => { svtcur.eit_pf[cnt].genre3 + 1 },
                };
    
                // 出力処理
                writeln!(outfile ,"  <{} start=\"{}\" stop=\"{}\" channel=\"{}\" eid=\"{}\">",
                    tag, cstarttime, cendtime, svtcur.ontv, svtcur.eit_pf[cnt].event_id).unwrap();
                writeln!(outfile, "    <title>{}</title>", title).unwrap();
                writeln!(outfile, "    <desc>{}</desc>", subtitle ).unwrap();
                writeln!(outfile, "    <genres>{}:{}:{}:{}:{}:{}</genres>",
                    content_type, svtcur.eit_pf[cnt].content_subtype,
                    genre2, svtcur.eit_pf[cnt].sub_genre2,
                    genre3, svtcur.eit_pf[cnt].sub_genre3).unwrap();
                writeln!(outfile, "    <video_audio>{}:{}:{}</video_audio>",
                    svtcur.eit_pf[cnt].video_type, svtcur.eit_pf[cnt].audio_type,
                    svtcur.eit_pf[cnt].multi_type).unwrap();
                writeln!(outfile, "    <status>{}</status>",
                    svtcur.eit_pf[cnt].event_status).unwrap();
                writeln!(outfile, "    <sch_pnt>{}</sch_pnt>",
                    svtcur.eit_pf[cnt].sch_pnt).unwrap();
                writeln!(outfile, "  </{}>", tag).unwrap();

            }
        }

        // eitschにデータがある場合の処理
        if svtcur.eitsch.len() > 0 {

            // eitsch配列分ループ
            for cnt in 0..svtcur.eitsch.len() {

                // ジャンル未定義補正処理呼び出し
                rest_repair(&mut svtcur.eitsch[cnt]);

                // 処理用タイトル作成
                let mut title = svtcur.eitsch[cnt].title.clone();
                title = xml_special_chars(title);

                // 処理用サブタイトル作成
                let mut subtitle = svtcur.eitsch[cnt].subtitle.clone();
                subtitle = xml_special_chars(subtitle);

                // 処理用タグ作成
                let tag = "programme".to_string();

                // 処理用開始日時作成
                let start_time = svtcur.eitsch[cnt].start_time;

                // 処理用終了日時作成
                let end_time = start_time + svtcur.eitsch[cnt].duration as i64;

                // 処理用終了日時(キャラクタ)作成
                let cendtime = Local.timestamp_opt(end_time,0).unwrap().format("%Y-%m-%d %H:%M:%S");

                // 処理用開始日時(キャラクタ)作成
                let cstarttime = format!("{:4}-{:02}-{:02} {:02}:{:02}:{:02}",
                    svtcur.eitsch[cnt].yy + 1900, svtcur.eitsch[cnt].mm, svtcur.eitsch[cnt].dd,
                    svtcur.eitsch[cnt].hh, svtcur.eitsch[cnt].hm, svtcur.eitsch[cnt].ss);

                // 処理用コンテンツタイプ作成
                let content_type = match svtcur.eitsch[cnt].content_type {
                    16 => { 0 },
                    _ => { svtcur.eitsch[cnt].content_type + 1 },
                };
                
                // 処理用genre2作成
                let genre2 = match svtcur.eitsch[cnt].genre2 {
                    16 => { 0 },
                    _ => { svtcur.eitsch[cnt].genre2 + 1 },
                };

                // 処理用genre3作成
                let genre3 = match svtcur.eitsch[cnt].genre3 {
                    16 => { 0 },
                    _ => { svtcur.eitsch[cnt].genre3 + 1 },
                };

                // 出力処理
                writeln!(outfile ,"  <{} start=\"{}\" stop=\"{}\" channel=\"{}\" eid=\"{}\">",
                    tag, cstarttime, cendtime, svtcur.ontv, svtcur.eitsch[cnt].event_id).unwrap();
                writeln!(outfile, "    <title>{}</title>", title).unwrap();
                writeln!(outfile, "    <desc>{}</desc>", subtitle ).unwrap();
                writeln!(outfile, "    <genres>{}:{}:{}:{}:{}:{}</genres>",
                    content_type, svtcur.eitsch[cnt].content_subtype,
                    genre2, svtcur.eitsch[cnt].sub_genre2,
                    genre3, svtcur.eitsch[cnt].sub_genre3).unwrap();
                writeln!(outfile, "    <video_audio>{}:{}:{}</video_audio>",
                    svtcur.eitsch[cnt].video_type, svtcur.eitsch[cnt].audio_type,
                    svtcur.eitsch[cnt].multi_type).unwrap();
                //writeln!(outfile, "    <status>{}</status>", svtcur.eitsch[cnt].event_status);
                //writeln!(outfile, "    <sch_pnt>{}</sch_pnt>", svtcur.eitsch[cnt].sch_pnt);
                writeln!(outfile, "  </{}>", tag).unwrap();

            }
        }

        // eit_pf、eitschにデータがある場合にフッター出力
        if svtcur.eit_pf.len() > 0 || svtcur.eitsch.len() > 0 {
            writeln!(outfile, "<programme_cnt><disc>{}</disc><pf_cnt>{}</pf_cnt><sch_cnt>{}</sch_cnt></programme_cnt>",
                svtcur.ontv, svtcur.eit_pf.len(), svtcur.eitsch.len()).unwrap();
        }
    }
}
