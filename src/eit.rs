extern crate chrono;

use chrono::{DateTime, Local, TimeZone};
use log::{error};
//use chrono::prelude::{Datelike, Timelike};
#[allow(unused_imports)]
use log::{debug, info};

use crate::arib::{arib_to_string};
use crate::{CommanLineOpt};
use crate::sdt::{service_id_cehck};
use crate::ts::{MAXSECLEN, EitControl, SvtControlTop};

// EITヘッダー構造体
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
struct EitHead {
    table_id: u32,
    section_syntax_indicator: i32,
    reserved_future_use: i32,
    reserved1: i32,
    section_length: i32,
    service_id: i32,
    reserved2: i32,
    version_number: i32,
    current_next_indicator: i32,
    section_number: i32,
    last_section_number: i32,
    transport_stream_id: u32,
    original_network_id: i32,
    segment_last_section_number: i32,
    last_table_id: u32,
}

// EITボディー構造体
#[derive(Debug, Copy, Clone)]
struct EitBody {
    event_id: i32,
    running_status: i32,
    free_ca_mode: i32,
    descriptors_loop_length: i32,
    // 以下は解析結果保存用
    yy: i32,
    mm: i32,
    dd: i32,
    hh: i32,
    hm: i32,
    ss: i32,
    duration: i32,
    start_time: i64,
    event_status: i32,
}

// イベント詳細構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SevtDesc {
    descriptor_tag: i32,
    descriptor_length: i32,
    iso_639_language_code: String,
    event_name_length: i32,
    event_name: String,
    text_length: i32,
    text: String,
}

// コンテンツ詳細構造体
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
struct ContentDesc {
    descriptor_tag: i32,
    descriptor_length: i32,
    content: [u8; MAXSECLEN],
}

// シリーズ詳細構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SeriesDesc {
    descriptor_tag: i32,
    descriptor_length: i32,
    series_id: i32,
    repeat_label: i32,
    program_pattern: i32,
    expire_date_valid_flag: i32,
    expire_date: i32,
    episode_number: i32,
    last_episode_number: i32,
    //series_name_char: [u8; MAXSECLEN],
    series_name_char: String,
}

// コンポーネント詳細構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ComponentDesc {
    descriptor_tag: i32,
    descriptor_length: i32,
    reserved_future_use: i32,
    stream_content: i32,
    component_type: i32,
    component_tag: i32,
    iso_639_language_code: String,
    text_char: String,
}

// オーディオコンポーネント詳細構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AudioComponentDesc {
    descriptor_tag: i32,
    descriptor_length: i32,
    reserved_future_use_1: i32,
    stream_content: i32,
    component_type: i32,
    component_tag: i32,
    stream_type: i32,
    simulcast_group_tag: i32,
    es_multi_lingual_flag: i32,
    main_component_flag: i32,
    quality_indicator: i32,
    sampling_rate: i32,
    reserved_future_use_2: i32,
    iso_639_language_code_1: String,
    iso_639_language_code_2: String,
    text_char: String,
}

// 拡張イベント記述子ヘッダー構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct EevtdHead {
    descriptor_tag: i32,
    descriptor_length: i32,
    descriptor_number: i32,
    last_descriptor_number: i32,
    iso_639_language_code: String,
    length_of_items: i32,
}

// 拡張イベント記述子アイテム構造体
#[derive(Debug, Clone)]
struct EevtdItem {
    item_description_length: i32,
    //item_description: [u8; MAXSECLEN],
    item_description: String,
    item_length: i32,
    //item: [u8; MAXSECLEN],
    item: String,
    //退避用
    descriptor_number: i32,
}

// 拡張イベント記述子終端構造体
#[derive(Debug, Clone)]
struct EevtdTail {
    text_length: i32,
    text: String,
}

// 定数定義
#[allow(dead_code)]
pub const CERTAINTY: i32 = 0x00;
#[allow(dead_code)]
pub const START_TIME_UNCERTAINTY: i32 = 0x01;
#[allow(dead_code)]
pub const DURATION_UNCERTAINTY: i32 = 0x02;
#[allow(dead_code)]
pub const EVENT_UNCERTAINTY: i32 = 0x03;
#[allow(dead_code)]
pub const NEXT_EVENT_UNCERTAINTY: i32 = 0x04;

//
// eittopへ追加、挿入処理
//
fn eittop_data_update(eittop: &mut Vec<EitControl>,
    eith: &EitHead, eitb: &EitBody, sevtd: &SevtDesc) -> () {


    // push_cnt初期化
    let mut push_cnt: i32 = -1;

    // データ挿入位置の確認
    for loop_cnt in 0..eittop.len() {
        if eittop[loop_cnt].start_time > eitb.start_time {
            push_cnt = loop_cnt as i32;
            break;
        }
    }

    // 最後に追加
    if push_cnt == -1 {

        // DateTime形式の時刻情報を作成
        let dt: DateTime<Local> = match Local.with_ymd_and_hms(
            eitb.yy as i32 + 1900, eitb.mm as u32, eitb.dd as u32,
            eitb.hh as u32, eitb.hm as u32, eitb.ss as u32).single() {
            Some(date_time) => date_time,
            None => {
                error!("日付変換エラー");
                return
            },
        };

        // データ更新
        eittop.push(EitControl {
            table_id: eith.table_id as i32,
            servid: eith.service_id,
            event_id: eitb.event_id,
            version_number: eith.version_number,
            section_number: eith.section_number,
            last_section_number: eith.last_section_number,
            segment_last_section_number: eith.segment_last_section_number,
            running_status: eitb.running_status,
            free_ca_mode: eitb.free_ca_mode,
            content_type: 0,
            content_subtype: 0,
            genre2: 0,
            sub_genre2: 0,
            genre3: 0,
            sub_genre3: 0,
            episode_number: 0,
            yy: eitb.yy,
            mm: eitb.mm,
            dd: eitb.dd,
            hh: eitb.hh,
            hm: eitb.hm,
            ss: eitb.ss,
            duration: eitb.duration,
            start_time: dt.timestamp(),
            title: sevtd.event_name.clone(),
            subtitle: sevtd.text.clone(),
            desc: String::new(),
            desc_length: 0,
            video_type: 0,
            audio_type: 0,
            multi_type: 0,
            event_status: eitb.event_status,
            sch_pnt: 0,
            import_cnt: 0,
            renew_cnt: 0,
            tid: 0,
            tid_status: 0,
        });
    }
    // 途中に追加
    else {

        // DateTime形式の時刻情報を作成
        let dt: DateTime<Local> = match Local.with_ymd_and_hms(
            eitb.yy as i32 + 1900, eitb.mm as u32, eitb.dd as u32,
            eitb.hh as u32, eitb.hm as u32, eitb.ss as u32).single() {
            Some(date_time) => date_time,
            None => {
                error!("日付変換エラー");
                return
            },
        };

        // データ更新
        eittop.insert(push_cnt as usize, EitControl {
            table_id: eith.table_id as i32,
            servid: eith.service_id,
            event_id: eitb.event_id,
            version_number: eith.version_number,
            section_number: eith.section_number,
            last_section_number: eith.last_section_number,
            segment_last_section_number: eith.segment_last_section_number,
            running_status: eitb.running_status,
            free_ca_mode: eitb.free_ca_mode,
            content_type: 0,
            content_subtype: 0,
            genre2: 0,
            sub_genre2: 0,
            genre3: 0,
            sub_genre3: 0,
            episode_number: 0,
            yy: eitb.yy,
            mm: eitb.mm,
            dd: eitb.dd,
            hh: eitb.hh,
            hm: eitb.hm,
            ss: eitb.ss,
            duration: eitb.duration,
            start_time: dt.timestamp(),
            title: sevtd.event_name.clone(),
            subtitle: sevtd.text.clone(),
            desc: String::new(),
            desc_length: 0,
            video_type: 0,
            audio_type: 0,
            multi_type: 0,
            event_status: eitb.event_status,
            sch_pnt: 0,
            import_cnt: 0,
            renew_cnt: 0,
            tid: 0,
            tid_status: 0,
        });

    };

}
//
// EIT編集処理
//
pub fn dump_eit(cmd_opt: &CommanLineOpt, buf: &[u8], mut svttop: &mut Vec<SvtControlTop>) -> () {

    // table_idポインタ変数
    let mut table_id_index = 0;

    // table_idが0xff以外でループ
    while buf[table_id_index] != 0xff {

        // EITヘッダーの取り込み
        let eith = EitHead {
            table_id: buf[table_id_index] as u32,
            section_syntax_indicator: buf[table_id_index + 1] as i32 & 0x80 >> 7,
            reserved_future_use: (buf[table_id_index + 1] as i32 & 0x40) >> 6,
            reserved1: (buf[table_id_index + 1] as i32 & 0x30) >> 4,
            section_length: ((buf[table_id_index + 1] as i32 & 0x0f) << 8) + buf[table_id_index + 2] as i32,
            service_id: ((buf[table_id_index + 3] as i32 & 0xff) << 8) + buf[table_id_index + 4] as i32,
            reserved2:((buf[table_id_index + 5] as i32 & 0xc0) >> 6),
            version_number: ((buf[table_id_index + 5] as i32 & 0x3e) >> 1),
            current_next_indicator: buf[table_id_index + 5] as i32 & 0x01,
            section_number: buf[table_id_index + 6] as i32,
            last_section_number: buf[table_id_index + 7] as i32,
            transport_stream_id: ((buf[table_id_index + 8] as u32 & 0xff) << 8) + buf[table_id_index + 9] as u32,
            original_network_id: ((buf[table_id_index + 10] as i32 &0xff) << 8) + buf[table_id_index + 11] as i32,
            segment_last_section_number: buf[table_id_index + 12] as i32,
            last_table_id:  buf[table_id_index + 13] as u32,
       };

        // 変数初期化
        let mut len = table_id_index + 14;
        let mut index: usize = 0;
        let eit_pf_flg = match eith.table_id {
            0x4e | 0x4f => true,
            _ => false, 
        };

        // 変数の作成
        let mut loop_len;
        let mut loop_blen;
        let mut loop_elen;
        let mut eittop: Vec<EitControl>;


        // EITモードがtrueでEIT PFフラッグがfalseの場合はリターン
        if cmd_opt.eit_mode == true && eit_pf_flg == false {

            return;

        };

        // SIDの指定があり指定されたSID以外の場合はリターン
        if cmd_opt.is_sid == true && cmd_opt.select_sid != eith.service_id {

            return;

        };

        // サービスIDの判定処理
        service_id_cehck(&cmd_opt, &mut svttop, eith.service_id);

        // 取得対象のサービス判定
        for cnt in 0..svttop.len() {

            // 取得対象サービスの処理
            if svttop[cnt].service_id == eith.service_id {

                svttop[cnt].svt_control_sub[0].service_id = eith.service_id;

                // EIT PFフラグによりClone元を変更
                if eit_pf_flg != true {

                    eittop = svttop[cnt].svt_control_sub[0].eitsch.clone();

                }
                else {

                    eittop = svttop[cnt].svt_control_sub[0].eit_pf.clone();

                };

                // 開始位置の設定
                index += len;

                // loopレングスの設定
                loop_len = eith.section_length - (14 - 3 + 4) as i32; // 3は共通ヘッダ長 4はCRC

                // loopレングスが0でEIT PFフラグがオフでかつ次セクションデータが無い場合はリターン
                if loop_len == 0 && eit_pf_flg == false && buf[index + 4] == 0xff {

                    return

                };

                // loopレングスが0以下になるまでループ
                while loop_len > 0 {

                    // 開始時間と時間長を設定
                    let start_time: &[u8] = &buf[index + 2..index + 2 + 5];
                    let duration: &[u8] =  &buf[index + 2 + 5..index + 2 + 5 + 3];

                    // 変数初期化
                    let mut save_eevtitem_item_length = 0;
                    let mut save_eevtitem_item: [u8; MAXSECLEN] = [0; MAXSECLEN];
                    let mut save_eevtitem = EevtdItem {
                        item_description_length: 0,
                        item_description: String::new(),
                        item_length: 0,
                        item: String::new(),
                        descriptor_number: 0,
                    };

                    // EITTボディー取り込み
                    let mut eitb = EitBody {
                        event_id: ((buf[index] as i32) << 8) + buf[index + 1] as i32,
                        running_status: (buf[index + 10] as i32 & 0xe0) >> 5,
                        free_ca_mode: (buf[index + 10] as i32 & 0x10) >> 4,
                        descriptors_loop_length: ((buf[index + 10] as i32 & 0x0f) << 8) + buf[index + 11] as i32,
                        yy: 0, mm: 0, dd: 0, hh: 0, hm: 0, ss: 0,
                        duration: 0,
                        start_time: 0,
                        event_status: 0,
                    };

                    // 日付変換
                    // オール0xffの処理
                    if start_time[0] == 0xff && start_time[1] == 0xff && start_time[2] == 0xff && 
                         start_time[3] == 0xff && start_time[4] == 0xff {

                        // eventステータスと年月を初期値で設定
                        eitb.event_status = START_TIME_UNCERTAINTY;
                        eitb.yy = 138;
                        eitb.mm = eith.section_number + 1;

                    }
                    // オール0xff以外の処理
                    else {

                        // 年、月、日の取り込み
                        let tnum = ((start_time[0] as i32) << 8) + start_time[1] as i32;
                        eitb.yy = ((tnum as f32 - 15078.2) / 365.25) as i32;
                        eitb.mm = (((tnum as f32 - 14956.1) - (eitb.yy as f32 * 365.25)) / 30.6001) as i32;
                        eitb.dd = ((tnum - 14956) - (eitb.yy as f32 * 365.25) as i32) - (eitb.mm as f32 * 30.6001) as i32;

                        // 月が14,15の場合に年、月を補正
                        if eitb.mm == 14 || eitb.mm == 15 {

                            eitb.yy += 1;
                            eitb.mm = eitb.mm - 1 - (1 * 12);

                        }
                        // 上記以外は月を-1
                        else {

                            eitb.mm -= 1;
    
                        };

                        // 時、分、秒の取り込み
                        eitb.hh = (((start_time[2] as i32) >> 4) * 10) + (start_time[2] as i32 & 0x0f);
                        eitb.hm = (((start_time[3] as i32) >> 4) * 10) + (start_time[3] as i32 & 0x0f);
                        eitb.ss = (((start_time[4] as i32) >> 4) * 10) + (start_time[4] as i32 & 0x0f);

                        // 時刻長が0以外時の処理
                        if duration[0] != 0x00 || duration[1] != 0x00 || duration[2] != 0x00 {

                            eitb.duration = (((duration[0] as i32) >> 4) * 10 + (duration[0] as i32 & 0x0f)) * 3600 +
                                (((duration[1] as i32) >> 4) * 10 + (duration[1] as i32 & 0x0f)) * 60 +
                                ((duration[2] as i32) >> 4) * 10 + (duration[2] as i32 & 0x0f);

                        };

                        // DateTime形式での日時、時刻情報の作成
                        let dt: DateTime<Local> = match Local.with_ymd_and_hms(
                            eitb.yy as i32 + 1900, eitb.mm as u32, eitb.dd as u32,
                            eitb.hh as u32, eitb.hm as u32, eitb.ss as u32).single() {
                            Some(date_time) => date_time,
                            None => {
                                error!("日付変換エラー");
                                return
                            },
                        };

                        // シリアル時刻の作成
                        eitb.start_time = dt.timestamp();

                    };

                    // カウンター更新
                    len = 12;
                    index += len;
                    loop_len -= len as i32;
                    loop_blen = eitb.descriptors_loop_length;
                    loop_len -= loop_blen;

                    // loop_blenが0以上の処理
                    while loop_blen > 0 {

                        // イベント詳細情報の取得
                        let mut sevtd = SevtDesc {
                            descriptor_tag: buf[index] as i32,
                            descriptor_length: buf[index + 1] as i32,
                            iso_639_language_code: String::new(),
                            event_name_length: 0,
                            event_name: String::new(),
                            text_length: 0,
                            text: String::new(),
                        };

                        // ディスクリプタータグが0x4dの場合はlenの設定と文字情報を取得
                        if buf[index] & 0xff == 0x4d {

                            // レングス取得
                            sevtd.descriptor_length = buf[index + 1] as i32;

                            // ランゲージコードの取得
                            sevtd.iso_639_language_code = match std::str::from_utf8(&buf[index + 2..index + 2 + 3]) {
                                Ok(lang_code) => lang_code.to_string(),
                                Err(_) => "jpn".to_string(),
                            };

                            // イベント名レングス、テキストレングスの退避
                            let work_event_name_length = buf[index + 5] as i32;
                            let work_text_length = buf[index + 6 + work_event_name_length as usize] as i32;

                            // イベント名長が0以上の処理
                            if work_event_name_length > 0 {

                                // イベント名の文字コード変換
                                (sevtd.event_name_length, sevtd.event_name) = 
                                    arib_to_string(&buf[index + 6..index + 6 + work_event_name_length as usize], work_event_name_length);

                            };

                            if work_text_length > 0 {

                                // イベントテキストの文字コード変換
                                (sevtd.text_length, sevtd.text) = 
                                    arib_to_string(&buf[index + 7 + work_event_name_length as usize..
                                        index + 7 + work_event_name_length as usize  + work_text_length as usize], work_text_length);

                            };

                            len = sevtd.descriptor_length as usize + 2;

                        }
                        // ディスクリプタータグが0x4d以外の場合はlenに0を設定
                        else {

                            len = 0;
    
                        };

                        // lenが0より大きい（文字情報あり）の処理
                        if len > 0 {

                            if eitb.event_status != EVENT_UNCERTAINTY {

                                let mut seach_flg = false;

                                // eittopがある分だけループ
                                for cnt2 in 0..eittop.len() {

                                    if eittop[cnt2].event_id == eitb.event_id && eittop[cnt2].servid == eith.service_id {

                                        // DateTime形式の時刻情報を作成
                                        let dt: DateTime<Local> = match Local.with_ymd_and_hms(
                                            eitb.yy as i32 + 1900, eitb.mm as u32, eitb.dd as u32,
                                            eitb.hh as u32, eitb.hm as u32, eitb.ss as u32).single() {
                                            Some(date_time) => date_time,
                                            None => {
                                                error!("日付変換エラー");
                                                return
                                            },
                                        };

                                        // 構造体の情報更新
                                        eittop[cnt2].version_number = eith.version_number;
                                        eittop[cnt2].section_number = eith.section_number;
                                        eittop[cnt2].last_section_number = eith.last_section_number;
                                        eittop[cnt2].segment_last_section_number = eith.segment_last_section_number;
                                        eittop[cnt2].running_status = eitb.running_status;
                                        eittop[cnt2].yy = eitb.yy;
                                        eittop[cnt2].mm = eitb.mm;
                                        eittop[cnt2].dd = eitb.dd;
                                        eittop[cnt2].hh = eitb.hh;
                                        eittop[cnt2].hm = eitb.hm;
                                        eittop[cnt2].ss = eitb.ss;
                                        eittop[cnt2].duration = eitb.duration;
                                        eittop[cnt2].start_time = dt.timestamp();
                                        eittop[cnt2].event_status = eitb.event_status;

                                        // サーチフラグ on
                                        seach_flg = true;

                                        // EIT PFフラグがfalseの場合にimport_cntのカウントアップ
                                        if eit_pf_flg != true {
                                            eittop[cnt2].import_cnt += 1;
                                            //eittop[cnt2].import_cnt = eittop[cnt2].import_cnt + 1;
                                            eittop[cnt2].renew_cnt += 1;
                                        };
                                    };
                                };

                                // 
                                if seach_flg == false {

                                    // eittop配列への追加、挿入処理呼出し
                                    eittop_data_update(&mut eittop, &eith, &eitb, &sevtd);
                                };
                            };
                        }
                        else {

                            // イベント情報ヘッダの取り込み
                            let mut eevthead = EevtdHead {
                                descriptor_tag: buf[index] as i32,
                                descriptor_length: buf[index + 1] as i32,
                                descriptor_number: 0,
                                last_descriptor_number: 0,
                                iso_639_language_code: String::new(),
                                length_of_items: 0,
                            };

                            // ディスクリプタータグが0x4eの場合はlenの設定と文字情報を取得
                            if (buf[index] & 0xff) == 0x4e {

                                // descriptor情報取得
                                eevthead.descriptor_length = buf[index + 1] as i32;
                                eevthead.descriptor_number = (buf[index + 2] as i32) >> 4;
                                eevthead.last_descriptor_number = buf[index + 2] as i32 & 0x0f;
                                eevthead.iso_639_language_code = match std::str::from_utf8(&buf[index + 3..index + 3 + 3]) {
                                    Ok(lang_code) => String::from(lang_code),
                                    Err(_) => String::from("jpn"),
                                };

                                eevthead.length_of_items = buf[index + 6] as i32;
                                len = 7;

                            }
                            // ディスクリプタータグが0x4e以外の場合はlenに0を設定
                            else {

                                len = 0;

                            }

                            // lenが0より大きい（文字情報あり）の処理
                            if len > 0 {

                                // カウンター更新
                                index += len;
                                loop_blen -= len as i32;
                                loop_elen = eevthead.length_of_items;

                                // loop_elenが0以下になるまでループ
                                while loop_elen > 0 {

                                    // 変数初期化
                                    let mut work_item_description_length = 0;
                                    let mut work_item: [u8; MAXSECLEN] = [0; MAXSECLEN];
                                    let work_item_length;

                                    // 拡張イベント情報の取得
                                    let mut eevtitem = EevtdItem {
                                        item_description_length: buf[index] as i32,
                                        item_description: String::new(),
                                        item_length: 0,
                                        item: String::new(),
                                        descriptor_number: 0,
                                    };

                                    // レングス設定
                                    work_item_length = buf[index + 1 + eevtitem.item_description_length as usize] as i32;
                                    len = eevtitem.item_description_length as usize + work_item_length as usize + 2;
    
                                    // アイテム詳細長が1以上の処理
                                    if eevtitem.item_description_length > 0 {

                                        // 文字コード変換
                                        (work_item_description_length, eevtitem.item_description) =
                                            arib_to_string(&buf[index + 1..index + 1 + eevtitem.item_description_length as usize],
                                                eevtitem.item_description_length);

                                    };

                                    // アイテム長が1以上の処理
                                    if work_item_length > 0 {

                                        // ワークアイテム変数に退避
                                        work_item[..work_item_length as usize]
                                            .copy_from_slice(&buf[index + 2 + eevtitem.item_description_length as usize..index + 2 +
                                                eevtitem.item_description_length as usize + work_item_length as usize]);
                                    
                                    };

                                    // アイテム詳細長の取得
                                    eevtitem.item_description_length = work_item_description_length;

                                    // カウンター更新
                                    index += len;
                                    loop_elen -= len as i32;
                                    loop_blen -= len as i32;
                                
                                    // アイテム詳細構造体のレングスが0の場合の処理
                                    if eevtitem.item_description_length == 0 {

                                        // セーブエリアの初期化
                                        if save_eevtitem_item_length == 0 {

                                            save_eevtitem_item = [0; MAXSECLEN]
    
                                        };

                                        // セーブエリアに退避
                                        save_eevtitem_item[save_eevtitem_item_length as usize..
                                            save_eevtitem_item_length as usize + work_item_length as usize]
                                            .copy_from_slice(&work_item[..work_item_length as usize]);
                                        save_eevtitem_item_length += work_item_length;

                                    }
                                    // アイテム詳細構造体のレングスが0以外の処理
                                    else {

                                        // セーブエリアのアイテムレングスが0以外の処理
                                        if save_eevtitem_item_length != 0 {

                                            // 文字コート変換
                                            (save_eevtitem.item_length, save_eevtitem.item) =
                                                arib_to_string(&save_eevtitem_item, save_eevtitem_item_length);
                                        
                                            // eittop配列数分処理
                                            for apent_cnt in 0..eittop.len() {

                                                // イベントＩＤとサービスＩＤが同じ場合の処理（既情報のアップデート）
                                                if eittop[apent_cnt].event_id == eitb.event_id && eittop[apent_cnt].servid == eith.service_id {

                                                    // 格納エリアを初期化
                                                    eittop[apent_cnt].desc = String::new();

                                                    // 文字情報を格納
                                                    eittop[apent_cnt].desc.push_str(&save_eevtitem.item_description);
                                                    eittop[apent_cnt].desc.push_str("\t");
                                                    eittop[apent_cnt].desc.push_str(&save_eevtitem.item);
                                                    eittop[apent_cnt].desc_length = eittop[apent_cnt].desc.len() as i32;
    
                                                };
                                            };

                                            // 退避エリア変数の作成
                                            let swap_eevtitem = eevtitem;

                                            // データ入れ替え
                                            save_eevtitem = swap_eevtitem;
                                            save_eevtitem.descriptor_number = eevthead.descriptor_number;
                                            save_eevtitem_item = work_item;
                                            save_eevtitem_item_length = work_item_length;

                                        }
                                        // セーブエリアのアイテムレングスが0の場合の処理
                                        else {

                                            // ワーク変数のセーブ
                                            save_eevtitem_item_length = work_item_length;
                                            save_eevtitem_item = work_item;
                                            save_eevtitem = eevtitem;

                                        };
                                    };
                                };

                                // 終端情報の取得
                                let mut eevttail = EevtdTail {
                                    text_length: buf[index] as i32,
                                    text: String::new(),
                                };
                                len = eevttail.text_length as usize + 1;

                                // 終端情報長が1以上の処理
                                if eevttail.text_length > 0 {

                                    // 文字コード変換
                                    (eevttail.text_length, eevttail.text) =
                                        arib_to_string(&buf[index + 1..index + 1 + eevttail.text_length as usize], eevttail.text_length);

                                };
                            }
                            // lenが0以下（文字情報なし）の処理
                            else {

                                // eittopが未設定の処理
                                if eittop.len() == 0 {

                                    let _descriptor_tag = buf[index] as i32;
                                    let descriptor_length = buf[index + 1] as i32;

                                    // カウンター更新
                                    len = descriptor_length as usize + 2;

                                };

                                // eittop配列数分ループ
                                for cnt2 in 0..eittop.len() {

                                    if eittop[cnt2].event_id == eitb.event_id && eittop[cnt2].servid == eith.service_id {

                                        // ディスクリプションタブ毎の処理
                                        match buf[index] {
                                            0x54 => {  // コンテンツ記述子

                                                // コンテンツ詳細の取得
                                                let mut content_desc = ContentDesc {
                                                    descriptor_tag: buf[index] as i32,
                                                    descriptor_length: buf[index + 1] as i32,
                                                    content: [0; MAXSECLEN],
                                                };
                                                content_desc.content[..content_desc.descriptor_length as usize]
                                                    .copy_from_slice(&buf[index + 2..index + 2 + content_desc.descriptor_length as usize]);
                                                len = content_desc.descriptor_length as usize + 2;

                                                // コンテンツ詳細長が1以上の処理
                                                if len > 0 {

                                                    // コンテンツタイプの取得
                                                    eittop[cnt2].content_type = content_desc.content[0] as i32 >> 4;
    
                                                    // コンテンツタイプが14以外の処理
                                                    if eittop[cnt2].content_type != 14 {

                                                        eittop[cnt2].content_subtype = content_desc.content[0] as i32 & 0x0f;

                                                    }
                                                    // コンテンツタイプが14の場合の処理
                                                    else {

                                                        // コンテンツサブタイプの取得(コンテンツ詳細によって変化)
                                                        eittop[cnt2].content_subtype =
                                                            if (content_desc.content[0] as u8 & 0x0f) == 0x01 {
                                                                content_desc.content[1] as i32 + 0x40
                                                            }
                                                            else {
    
                                                                content_desc.content[1] as i32

                                                            };
                                                    };

                                                    // コンテンツ詳細のディスクリプター長が4以上の処理
                                                    if content_desc.descriptor_length >= 4 {

                                                        // コンテンツgenre2の取得
                                                        eittop[cnt2].genre2 = content_desc.content[2] as i32 >> 4;

                                                        // コンテンツgenre2が14以外の処理
                                                        if eittop[cnt2].genre2 != 14 {

                                                            // コンテンツサブgenre2の取得
                                                            eittop[cnt2].sub_genre2 = content_desc.content[2] as i32 & 0x0f;

                                                        }
                                                        else {

                                                            // コンテンツサブgenre2の取得(コンテンツ詳細によって変化)
                                                            eittop[cnt2].sub_genre2 =
                                                                if (content_desc.content[2] as u8 & 0x0f) == 0x01 {

                                                                    content_desc.content[3] as i32 + 0x40

                                                                }
                                                                else {

                                                                    content_desc.content[3] as i32

                                                                };
                                                        };

                                                        // コンテンツ詳細のディスクリプター長が6以上の処理
                                                        if content_desc.descriptor_length >= 6 {

                                                            // コンテンツgenre3の取得
                                                            eittop[cnt2].genre3 = content_desc.content[4] as i32 >> 4;

                                                            // コンテンツgenre3が14以外の処理
                                                            if eittop[cnt2].genre3 != 14 {

                                                                // コンテンツサブgenre3の取得
                                                                eittop[cnt2].sub_genre3 = content_desc.content[4] as i32 & 0x0f;

                                                            }
                                                            else {

                                                                // コンテンツサブgenre3の取得(コンテンツ詳細によって変化)
                                                                eittop[cnt2].sub_genre3 =
                                                                    if (content_desc.content[4] as u8 & 0x0f) == 0x01 {

                                                                        content_desc.content[5] as i32 & 0x40

                                                                    }
                                                                    else {
    
                                                                        content_desc.content[5] as i32

                                                                    };
                                                            };
                                                        }
                                                        else {

                                                            // コンテンツgenre3、コンテンツサブgenre3の設定
                                                            eittop[cnt2].genre3 = 16;
                                                            eittop[cnt2].sub_genre3 = 16;

                                                        };

                                                        // コンテンツタイプが14の場合の処理
                                                        if eittop[cnt2].content_type == 14 {

                                                            // コンテンツサブタイプの退避
                                                            let sub_stock = eittop[cnt2].content_subtype;

                                                            // コンテンツgenre2が14以外の処理
                                                            if eittop[cnt2].genre2 != 14 {

                                                                eittop[cnt2].content_type = eittop[cnt2].genre2;
                                                                eittop[cnt2].content_subtype = eittop[cnt2].sub_genre2;
                                                                eittop[cnt2].genre2 = 14;
                                                                eittop[cnt2].sub_genre2 = sub_stock;

                                                            }
                                                            // コンテンツgenre2が14、16以外の処理
                                                            else if eittop[cnt2].genre3 != 14 && eittop[cnt2].genre3 != 16 {

                                                                eittop[cnt2].content_type = eittop[cnt2].genre3;
                                                                eittop[cnt2].content_subtype = eittop[cnt2].sub_genre3;
                                                                eittop[cnt2].genre3 = 14;
                                                                eittop[cnt2].sub_genre3 = sub_stock;

                                                            };
                                                        };
                                                    }
                                                    else {

                                                        eittop[cnt2].genre2     = 16;
                                                        eittop[cnt2].sub_genre2 = 16;
                                                        eittop[cnt2].genre3     = 16;
                                                        eittop[cnt2].sub_genre3 = 16;

                                                    };
                                                }
                                            },
                                            0xd5 => {  // シリーズ記述子
                                            
                                                // シリーズ詳細の取得
                                                let mut series_desc = SeriesDesc {
                                                    descriptor_tag: buf[index] as i32,
                                                    descriptor_length: buf[index + 1] as i32,
                                                    series_id: ((buf[index + 2] as i32) << 8) + buf[index + 3] as i32,
                                                    repeat_label: (buf[index + 4] as i32 & 0xf0) >> 4,
                                                    program_pattern: (buf[index + 4] as i32 & 0x0e) >> 1,
                                                    expire_date_valid_flag: buf[index + 4] as i32 & 0x01,
                                                    expire_date: ((buf[index + 5] as i32) << 8) + buf[index + 6] as i32,
                                                    episode_number: ((buf[index + 7] as i32) << 8) +
                                                        ((buf[index + 8] as i32 & 0xf0) >> 4),
                                                    last_episode_number: ((buf[index + 8] as i32 & 0x0f) << 8) + 
                                                        buf[index + 9] as i32,
                                                    series_name_char: String::new(),
                                                };
                                                len = series_desc.descriptor_length as usize + 2;

                                                // シリーズ詳細長が9以上の処理
                                                if series_desc.descriptor_length > 8 {

                                                    (_, series_desc.series_name_char) = 
                                                        arib_to_string(&buf[index + 10..series_desc.descriptor_length as usize - 1],
                                                            series_desc.descriptor_length - 10);

                                                };

                                                // lenが1以上の処理
                                                if len > 0 {

                                                    eittop[cnt2].episode_number = series_desc.episode_number;

                                                };
                                            },
                                            0x50 => {  // コンポーネント記述子

                                                // コンポーネント情報の取得
                                                let mut component_desc = ComponentDesc {
                                                    descriptor_tag: buf[index] as i32,
                                                    descriptor_length: buf[index + 1] as i32,
                                                    reserved_future_use: (buf[index + 2] as i32 & 0xf0) >> 4,
                                                    stream_content: buf[index + 2] as i32 & 0x0f,
                                                    component_type: buf[index + 3] as i32,
                                                    component_tag: buf[index + 4] as i32,
                                                    iso_639_language_code: match std::str::from_utf8(&buf[index + 5..index + 5 + 3]) {
                                                        Ok(lang_code) => String::from(lang_code),
                                                        Err(_) => String::from("jpn"),
                                                    },
                                                    text_char: String::new(),
                                                };
                                                len = component_desc.descriptor_length as usize + 2;
    
                                                // コンポーネント情報が7以上の処理
                                                if component_desc.descriptor_length > 6 {

                                                    // 文字コード変換
                                                    (_, component_desc.text_char) =
                                                        arib_to_string(&buf[index + 8..index + 8 + component_desc.descriptor_length as usize - 1],
                                                            component_desc.descriptor_length - 6);

                                                }

                                                // lenが1以上の処理
                                                if len > 0 {

                                                    eittop[cnt2].video_type = component_desc.component_type;

                                                };
                                            },
                                            0xc4 => {  // オーディオコンポーネント記述子

                                                // オーディオコンポーネント情報の取得
                                                let mut audio_component_desc = AudioComponentDesc {
                                                    descriptor_tag: buf[index] as i32,
                                                    descriptor_length: buf[index + 1] as i32,
                                                    reserved_future_use_1: (buf[index + 2] as i32 & 0xf0) >> 4,
                                                    stream_content: buf[index + 2] as i32 & 0x0f,
                                                    component_type: buf[index + 3] as i32,
                                                    component_tag: buf[index + 4] as i32,
                                                    stream_type: buf[index + 5] as i32,
                                                    simulcast_group_tag: buf[index + 6] as i32,
                                                    es_multi_lingual_flag: (buf[index + 7] as i32 & 0x80) >> 7,
                                                    main_component_flag: (buf[index + 7] as i32 & 0x40) >> 6,
                                                    quality_indicator: (buf[index + 7] as i32 & 0x30) >> 4,
                                                    sampling_rate: (buf[index + 7] as i32 & 0x0e) >> 1,
                                                    reserved_future_use_2: buf[index + 7] as i32 & 0x01,
                                                    iso_639_language_code_1: match std::str::from_utf8(&buf[index + 8..index + 8 + 3]) {
                                                        Ok(lang_code) => String::from(lang_code),
                                                        Err(_) => String::from("jpn"),
                                                    },
                                                    iso_639_language_code_2: String::new(),
                                                    text_char: String::new(),
                                                };
                                                len = audio_component_desc.descriptor_length as usize + 2;

                                                // オーディオコンポーネントマルチランゲージフラグが1の処理
                                                if audio_component_desc.es_multi_lingual_flag == 1 {

                                                    // ランゲージコードの取得
                                                    audio_component_desc.iso_639_language_code_2 = match
                                                        std::str::from_utf8(&buf[index + 11..index + 11 + 3]) {
                                                            Ok(lang_code) => String::from(lang_code),
                                                            Err(_) => String::from("jpn"),
                                                    };

                                                    // オーディオコンポーネント長が15以上の処理
                                                    if audio_component_desc.descriptor_length > 14 {

                                                        // 文字コード変換
                                                        (_, audio_component_desc.text_char) =
                                                            arib_to_string(&buf[index + 14..
                                                                index + 14 + audio_component_desc.descriptor_length as usize - 1],
                                                                audio_component_desc.descriptor_length - 12);
    
                                                    };
                                                }
                                                // オーディオコンポーネントマルチランゲージフラグが1以外の処理
                                                else {

                                                    // オーディオコンポーネント長が12以上の処理
                                                    if audio_component_desc.descriptor_length > 11 {

                                                        // 文字コード変換
                                                        (audio_component_desc.descriptor_length, audio_component_desc.text_char) =
                                                            arib_to_string(&buf[index + 11..
                                                                index + 11 + audio_component_desc.descriptor_length as usize - 1],
                                                                audio_component_desc.descriptor_length - 9);

                                                        };
                                                };

                                                // lenが1以上の処理
                                                if len > 0 {

                                                    eittop[cnt2].audio_type = audio_component_desc.component_type;
                                                    eittop[cnt2].multi_type = audio_component_desc.es_multi_lingual_flag;
    
                                                };
                                            },
                                            _ => {  // 上記以外
                                                
                                                // データポインター移動
                                                let _descriptor_tag = buf[index] as i32;
                                                let descriptor_length = buf[index + 1] as i32;

                                                // カウンター更新
                                                len = descriptor_length as usize + 2;

                                            },
                                        }; 

                                        break;

                                    }
                                    else if cnt2 == eittop.len() - 1 {

                                        // データポインター移動
                                        let _descriptor_tag = buf[index] as i32;
                                        let descriptor_length = buf[index + 1] as i32;
                                    
                                        // カウンター更新
                                        len = descriptor_length as usize + 2;
                                        //loop_blen -= len as i32;

                                    };
                                };
                            };
                        };

                        // カウンター更新
                        index += len;
                        loop_blen -= len as i32;

                    };

                    // セーブ情報の最終処理
                    if save_eevtitem_item_length > 0 {

                        // 文字コード変換
                        (save_eevtitem.item_length, save_eevtitem.item) =
                            arib_to_string(&save_eevtitem_item, save_eevtitem_item_length);

                        // eittop配列数分処理
                        for apent_cnt in 0..eittop.len() {

                            // イベントＩＤとサービスＩＤが同じ場合の処理（既情報のアップデート）
                            if eittop[apent_cnt].event_id == eitb.event_id && eittop[apent_cnt].servid == eith.service_id {

                                // 格納エリアの初期化
                                eittop[apent_cnt].desc = String::new();

                                // データ更新
                                eittop[apent_cnt].desc.push_str(&save_eevtitem.item_description);
                                eittop[apent_cnt].desc.push_str("\t");
                                eittop[apent_cnt].desc.push_str(&save_eevtitem.item);
                                eittop[apent_cnt].desc_length = eittop[apent_cnt].desc.len() as i32;

                            };
                        };
                    };
                };

                // EIT PFフラグによりClone先を変更
                if eit_pf_flg != true {

                    svttop[cnt].svt_control_sub[0].eitsch = eittop.clone();

                }
                else {

                    svttop[cnt].svt_control_sub[0].eit_pf = eittop.clone();

                };
            };
        };

        table_id_index += eith.section_length as usize + 3;

    };
}
