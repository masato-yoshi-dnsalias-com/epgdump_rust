
use log::{debug};

use crate::arib::{arib_to_string};
use crate::{CommanLineOpt};
use crate::ts::{MAXSECLEN, SvtControl, SvtControlTop};

// SDTヘッダー
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
struct SdtHead {
    table_id: u32,
    section_syntax_indicator: i32,
    reserved_future_use1: i32,
    reserved1: i32,
    section_length: i32,
    transport_stream_id: u32,
    reserved2: i32,
    version_number: i32,
    current_next_indicator: i32,
    section_number: i32,
    last_section_number: i32,
    original_network_id: i32,
    reserved_future_use2: i32,
}

// SDTボディー
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
struct SdtBody {
    service_id: i32,
    reserved_future_use1: i32,
    eit_user_defined_flags: i32,
    eit_schedule_flag: i32,
    eit_present_following_flag: i32,
    running_status: i32,
    free_ca_mode: i32,
    descriptors_loop_length: i32,
}

// サービス詳細
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SvcDecs {
    descriptor_tag: i32,
    descriptor_length: i32,
    service_type: i32,
    service_provider_name_length: i32,
    //service_provider_name: [u8; MAXSECLEN],
    service_provider_name: String,
    service_name_length: i32,
    //service_name: [u8; MAXSECLEN],
    service_name: String,
}

// ロゴ詳細
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
struct LogDecs {
    descriptor_tag: i32,
    descriptor_length: i32,
    logo_transmission_type: i32,
    reserved_future_use1: i32,
    logo_id: i32,
    reserved_future_use2: i32,
    logo_version: i32,
    download_data_id: i32,

    logo_char: [u8; MAXSECLEN],
}

// ロゴ
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
struct Logo {
    logo: [u8; 4096],
    logo_size: i32,
}

//
// SvtControlTopをservice_idでチェックし、対象service_idが未登録の場合に追加する処理
//
pub fn service_id_cehck(cmd_opt: &CommanLineOpt, svttop: &mut Vec<SvtControlTop>, service_id: i32) -> () {

    // cut_sidの作成
    let cut_sid: Vec<&str> = if cmd_opt.cut_sid_list != "" {

        cmd_opt.cut_sid_list.split(',').collect()

    }
    else {

        vec![]

    };

    // cut対象service_id判定
    let mut found_cut_service_id_flag: bool = false;
    for cnt in 0..cut_sid.len() {
        if cut_sid[cnt].parse::<i32>().unwrap() == service_id {

            found_cut_service_id_flag = true;

            break;

        };
    }

    // 出力対象service_id判定
    let mut found_service_id_flag: bool = false;
    for cnt in 0..svttop.len() {
        if svttop[cnt].service_id == service_id {

            found_service_id_flag = true;
     
            break;

        };
    }

    // 出力対象service_idの追加処理
    if found_service_id_flag == false && found_cut_service_id_flag == false && cmd_opt.is_sid == false {

        // データ追加位置の取得
        let mut push_cnt: i32 = -1;
        for cnt in 0..svttop.len() {
            if service_id < svttop[cnt].service_id {

                push_cnt = cnt as i32;

                break;

            }
        }

        // 検索対象のservice_idより大きな値がなかったので後方に追加
        if push_cnt == -1 {

            debug!("add push svttop[{}].service_id={}", push_cnt, service_id);

            svttop.push(SvtControlTop{
                service_id: service_id,
               svt_control_sub: vec![],
            });

            push_cnt = svttop.len() as i32 - 1;

        }
        // 検索対象のservice_idより大きな値があったので途中に追加
        else {

            debug!("add insert svttop.[{}]service_id={}", push_cnt, service_id);
            svttop.insert(push_cnt as usize, SvtControlTop {
                service_id: service_id,
               svt_control_sub: vec![],
            });
        }

        // 追加したデータの値を設定
        svttop[push_cnt as usize].svt_control_sub.push(SvtControl {
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
    }
}

//
// サービスタイプ取得処理
//
fn stat_service_type(service_type: i32, service_id: i32, mode: bool) -> i32 {
/*
サービス形式種別
0x00 未定義
0x01 デジタルＴＶサービス
0x02 デジタル音声サービス
0x03 - 0x7F 未定義
0x80 - 0xA0 事業者定義
0xA1 臨時映像サービス
0xA2 臨時音声サービス
0xA3 臨時データサービス
0xA4 エンジニアリングサービス
0xA5 プロモーション映像サービス
0xA6 プロモーション音声サービス
0xA7 プロモーションデータサービス
0xA8 事前蓄積用データサービス
0xA9 蓄積専用データサービス
0xAA ブックマーク一覧データサービス
0xAB サーバー型サイマルサービス
0xAC 独立ファイルサービス
0xAD - 0xBF 未定義（標準化機関定義領域）
0xC0 データサービス（ワンセグも）
0xC1 - 0xFF 未定義
*/

    // サービスタイプの判定
    let result = match service_type {
        // データサービス
        0xc0 => {
            if mode == false && service_id != 910 { -2 }
            else { 2 }
        },
        // デジタルＴＶサービス,デジタル音声サービス
        0x01 | 0x02 => { 2 },
        // デフォルト
        _ => {
            if mode == false { -2 }
            else { 2 }
        },
    };

    result

}

//
// SDTの解析処理
//
pub fn dump_sdt(cmd_opt: &CommanLineOpt, buf: &[u8], mut svttop: &mut Vec<SvtControlTop>) -> () {

    // SDTヘッダー初期化
    let sdth = SdtHead {
        table_id: buf[0] as u32,
        section_syntax_indicator: buf[1] as i32 & 0x80 >> 7,
        reserved_future_use1: (buf[1] as i32 & 0x40) >> 6,
        reserved1: (buf[1] as i32 & 0x30) >> 4,
        section_length: ((buf[1] as i32 & 0x0f) << 8) + buf[2] as i32,
        transport_stream_id: ((buf[3] as u32 & 0xff) << 8) + buf[4] as u32,
        reserved2: ((buf[5] as i32 & 0xc0) >> 6),
        version_number: ((buf[5] as i32 & 0x3e) >> 1),
        current_next_indicator: buf[5] as i32 & 0x01,
        section_number: buf[6] as i32,
        last_section_number: buf[7] as i32,
        original_network_id: ((buf[8] as i32 &0xff) << 8) + buf[9] as i32,
        reserved_future_use2: buf[10] as i32,
    };

    // 変数初期化
    let ontvheader;
    let mut len = 11;
    let mut desc_len;
    let mut loop_len = sdth.section_length - (len - 3 + 4); // 3は共通ヘッダ長 4はCRC
    let mut index: usize = 0;
    index += len as usize;

    // ontvheaderの初期化
    if cmd_opt.is_bs == true {

        ontvheader = "BS".to_string();

    }
    else if cmd_opt.is_cs == true {

        ontvheader = "CS".to_string();

    }
    else {

        ontvheader = cmd_opt.id.clone();

    };

    // loopレングスが0以下になるまでループ
    while loop_len > 0 {

        // SDTボディー取り込み
        let sdtb = SdtBody {
            service_id: ((buf[index] as i32) << 8) + buf[index + 1] as i32,
            reserved_future_use1: ((buf[index + 2] as i32 & 0xe0) >> 5),
            eit_user_defined_flags: ((buf[index + 2] as i32 & 0x01) >> 2) + ((buf[index + 2] as i32 & 0xc0) >> 2),
            eit_schedule_flag: ((buf[index + 2] as i32 & 0x02) >> 1),
            eit_present_following_flag: buf[index + 2] as i32 & 0x01,
            running_status: ((buf[index + 3] as i32 & 0xe0) >> 5),
            free_ca_mode: ((buf[index + 3] as i32 & 0x10) >> 4),
            descriptors_loop_length: ((buf[index + 3] as i32 & 0x0f) << 8) + buf[index + 4] as i32,
        };
        len = 5;

        // カウンター変更
        index += len as usize;
        loop_len -= len;
        desc_len = sdtb.descriptors_loop_length;
        loop_len -= desc_len;

        // 詳細レングスが0以下になるまでループ
        while desc_len > 0 {

            // テーブルIDが0xcfの処理
            if buf[index] == 0xcf {

                // ロゴ詳細初期化
                let mut logd = LogDecs {
                    descriptor_tag: buf[index] as i32,
                    descriptor_length: buf[index + 1] as i32,
                    logo_transmission_type: buf[index + 2] as i32,
                    reserved_future_use1: ((buf[index + 3] as i32 & 0xfe) >> 1),
                    logo_id: 0,
                    reserved_future_use2: 0,
                    logo_version: 0,
                    download_data_id: 0,
                    logo_char: [0; MAXSECLEN],
                };

                // ロゴ転送タイプの判定
                match logd.logo_transmission_type {
                    // 0x01 CDT 伝送方式1:CDT をダウンロードデータ識別で直接参照する場合
                    0x01 => {

                        logd.logo_id = ((buf[index + 3] as i32 & 0x01) << 8) + buf[index + 4] as i32;
                        logd.reserved_future_use2 = (buf[index + 5] as i32 & 0xf0) >> 4;
                        logd.logo_version = ((buf[index + 5] as i32 & 0x0f) << 8) +  buf[index + 6] as i32;
                        logd.download_data_id = ((buf[index + 7] as i32 & 0xff) << 8) + buf[index + 8] as i32;

                    },
                    // 0x02 CDT 伝送方式2:CDT をロゴ識別を用いてダウンロードデータ識別を間接的に参照する場合
                    0x02 => {

                        logd.logo_id = ((buf[index + 3] as i32 & 0x01) << 8) + buf[index + 4] as i32;
                        logd.reserved_future_use2 = (buf[index + 5] as i32 & 0xf0) >> 4;

                    },
                    // 0x03 簡易ロゴ方式
                    0x03 => {

                        logd.logo_char[..logd.descriptor_length as usize]
                            .copy_from_slice(&buf[index + 2..index + 2 + logd.descriptor_length as usize]);

                    },
                    _ => {},
                };

                len = logd.descriptor_length + 2;
                index += len as usize;
                desc_len -= len;

                // ゴ転送タイプが0x01以外の場合は次ループ継続
                if logd.logo_transmission_type != 0x01 { continue; };

                // ロゴを構造体に転送
                for cnt in 0..=svttop.len() - 1 {
                    if svttop[cnt].service_id == sdtb.service_id {

                        svttop[cnt].svt_control_sub[0].logo_download_data_id = logd.download_data_id as u32;
                        svttop[cnt].svt_control_sub[0].logo_version = logd.logo_version as u32;

                        break;

                    };
                };

                continue;

            }
            // テーブルインデックスが0x48以外の処理
            else if buf[index] != 0x48 {

                // ポインターを移動
                len = buf[index + 1] as i32 + 2;
                index += len as usize;
                desc_len -= len;
                continue;

            };

            // サービス詳細初期化
            let mut desc = SvcDecs {
                descriptor_tag: buf[index] as i32,
                descriptor_length: buf[index + 1] as i32,
                service_type: buf[index + 2] as i32,
                service_provider_name_length: buf[index + 3] as i32,
                service_provider_name: String::new(),
                service_name_length: 0,
                service_name: String::new(),
            };

            // サービスプロバイダー名長が1以上の場合に構造体へ転送
            if desc.service_provider_name_length > 0 {

                // 文字コード変換
                (desc.service_provider_name_length, desc.service_provider_name) =
                    arib_to_string(&buf[index + 4..index + 4 + desc.service_provider_name_length as usize], desc.service_provider_name_length); 

            };
            desc.service_name_length = buf[index + 4 + desc.service_provider_name_length as usize] as i32;

            // サービス名長が1以上の場合に構造体へ転送
            if desc.service_name_length > 0 {
                (desc.service_name_length, desc.service_name) = 
                    arib_to_string(&buf[index + 5 + desc.service_provider_name_length as usize..
                    index + 5 + desc.service_provider_name_length as usize + desc.service_name_length as usize], desc.service_name_length);
            };
            len = desc.descriptor_length + 2;
            index += len as usize;
            desc_len -= len;

            // サービスIDの判定処理
            service_id_cehck(&cmd_opt, &mut svttop, sdtb.service_id);

            // 取得対象のサービス判定
            for cnt in 0..svttop.len() {

                // 取得対象サービスの処理
                if svttop[cnt].service_id == sdtb.service_id {
                    
                    // 構造体が未初期化の場合に初期化
                    if svttop[cnt].svt_control_sub[0].service_id == 0 {

                        svttop[cnt].svt_control_sub[0].service_id = sdtb.service_id;
                        svttop[cnt].svt_control_sub[0].service_type = desc.service_type;
                        svttop[cnt].svt_control_sub[0].original_network_id = sdth.original_network_id;
                        svttop[cnt].svt_control_sub[0].transport_stream_id = sdth.transport_stream_id;
                        svttop[cnt].svt_control_sub[0].servicename = desc.service_name;
                        svttop[cnt].svt_control_sub[0].ontv = format!("{} {}", ontvheader, sdtb.service_id);

                        // サービスタイプの設定
                        svttop[cnt].svt_control_sub[0].import_stat = stat_service_type(desc.service_type, sdtb.service_id, cmd_opt.sdt_mode);

                        debug!("new svttop[{}].svt_control_sub[0]={:?}", cnt, svttop[cnt].svt_control_sub[0]);
                        
                    }
                    // サービスタイプが未設定の場合に設定
                    else if svttop[cnt].svt_control_sub[0].import_stat == 0 {

                        svttop[cnt].svt_control_sub[0].service_type = desc.service_type;
                        svttop[cnt].svt_control_sub[0].original_network_id = sdth.original_network_id;
                        svttop[cnt].svt_control_sub[0].transport_stream_id = sdth.transport_stream_id;
                        svttop[cnt].svt_control_sub[0].servicename = desc.service_name;
                        svttop[cnt].svt_control_sub[0].ontv = format!("{}_{}", ontvheader, sdtb.service_id);
                        svttop[cnt].svt_control_sub[0].import_stat = stat_service_type(desc.service_type, sdtb.service_id, cmd_opt.sdt_mode);

                        debug!("svttop[cnt].svt_control_sub[0].import_stat={}, svttop[{}].svt_control_sub[0]={:?}",
                            svttop[cnt].svt_control_sub[0].import_stat, cnt, svttop[cnt].svt_control_sub[0]);

                    }
                    // サービスタイプが-1の場合に設定
                    else if svttop[cnt].svt_control_sub[0].import_stat == -1 {
                        svttop[cnt].svt_control_sub[0].service_type = desc.service_type;
                        svttop[cnt].svt_control_sub[0].original_network_id = sdth.original_network_id;
                        svttop[cnt].svt_control_sub[0].transport_stream_id = sdth.transport_stream_id;
                        svttop[cnt].svt_control_sub[0].servicename = desc.service_name;
                        svttop[cnt].svt_control_sub[0].ontv = format!("{} {}", ontvheader, sdtb.service_id);
                        svttop[cnt].svt_control_sub[0].import_stat = 1;

                        debug!("svttop[cnt].svt_control_sub[0].import_stat={}, svttop[{}].svt_control_sub[0]={:?}",
                            svttop[cnt].svt_control_sub[0].import_stat, cnt, svttop[cnt].svt_control_sub[0]);

                    };

                    break;

                };
            };
        }
    };
}
