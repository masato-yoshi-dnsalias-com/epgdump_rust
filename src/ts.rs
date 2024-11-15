use std::fs::File;
use std::io::{BufRead, BufReader};

// 
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub struct DsmlControl {
    pub is_used: i32,
    pub module_id: i32,
    pub last_block_number: i32,
    pub block_size: i32,
    pub block_data: u32,
}

// MPEG2-TSパケット構造体
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub struct TsPacket {
    pub sync: u8,
    pub transport_error_indicator: i32,
    pub payload_unit_start_indicator: i32,
    pub transport_priority: i32,
    pub pid: u32,
    pub transport_scrambling_control: i32,
    pub adaptation_field_control: i32,
    pub continuity_counter: i32,
    pub adaptation_field: i32,
    pub payload: [u8; TSPAYLOADMAX],
    pub payloadlen: i32,
    pub rcount: i32,
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub struct SecCache {
    pub pid: u32,
    pub buf: [u8; MAXSECLEN],
    pub seclen: i32,
    pub setlen: i32,
    pub cur: TsPacket,
    pub curlen: i32,
    pub cont: i32
}

// EIT情報構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EitControl {
    pub table_id: i32,
    pub servid: i32,
    pub event_id: i32,                     // イベントID
    pub version_number: i32,
    pub section_number: i32,
    pub last_section_number: i32,
    pub segment_last_section_number: i32,
    pub running_status: i32,
    pub free_ca_mode: i32,
    pub content_type: i32,                 // コンテントタイプ
    pub content_subtype: i32,              // コンテントサブタイプ
    pub genre2: i32,
    pub sub_genre2: i32,
    pub genre3: i32,
    pub sub_genre3: i32,
    pub episode_number: i32,
    pub yy: i32,
    pub mm: i32,
    pub dd: i32,
    pub hh: i32,
    pub hm: i32,
    pub ss: i32,
    pub duration: i32,
    pub start_time: i64,
    pub title: String,                     // タイトル
    pub subtitle: String,                  // サブタイトル
    pub desc: String,                      // 詳細説明
    pub desc_length: i32,                  // 詳細説明のレングス
    pub video_type: i32,                   // 映像のタイプ
    pub audio_type: i32,                   // 音声のタイプ
    pub multi_type: i32,                   // 音声の 2 カ国語多重
    pub event_status: i32,
    pub sch_pnt: i32,
    pub import_cnt: i32,
    pub renew_cnt: i32,                    // 更新カウンタ
    pub tid: i32,
    pub tid_status: i32,
}

// サービス情報構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SvtControl {
    pub service_id: i32,
    pub service_type: i32,
    pub original_network_id: i32,
    pub transport_stream_id: u32,
    pub slot: i32,
    pub servicename: String,
    pub ontv: String,
    pub eitsch: Vec<EitControl>,
    pub eit_pf: Vec<EitControl>,
    pub prev_sch: Vec<EitControl>,
    pub import_cnt: i32,
    pub import_stat: i32,
    pub logo_download_data_id: u32,
    pub logo_version: u32,
}

// サービス情報トップ構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SvtControlTop {
    pub service_id: i32,
    //pub svt_control_sub: Option<Box<Vec<SvtControl>>>,
    pub svt_control_sub: Vec<SvtControl>,
}

/*
// EIT nullセグメント構造体
#[derive(Debug, Clone)]
//#[allow(dead_code)]
pub struct EitNullSegment {
    table_id: i32,
    service_id: i32,
    section_number: i32,
    version_number: i32,
}

// EIT nullセグメントトップ構造体
#[derive(Debug, Clone)]
//#[allow(dead_code)]
pub struct EitNullSegmentTop {
    pub service_id: i32,
    pub eit_null_segment_sub: Vec<EitNullSegment>,
}
*/

// 定数設定
pub const MAXSECLEN: usize = 4096;    // SEC構造体最大長
pub const TSPAYLOADMAX: usize = 184;  // 最大ペイロード長
pub const LENGTH_PACKET: usize = 188; // 最大パケット長

//
// TSパケットリード処理
//
pub fn read_ts(readbuff_file: &mut BufReader<&File>, secs: &mut [SecCache], count: usize) -> Option<SecCache> {

    // 変数の作成と初期化
    static RCOUNT: i32 = 0;     // パケットリードカウンター

    let mut tpk: TsPacket;      // MPEG-TSパケット
    let mut payptr = 4;         // ペイロードポインター
    let mut _index: usize = 0;  // インデックスカウンター

    loop {

        // ファイルリード処理
        let length = {

            // ファイルリードしバッファーに格納
            let read_buffer = readbuff_file.fill_buf().unwrap();
            
            // レングスが０以下の場合は当該パケットをスキップ
            let len = read_buffer.len();
            if len <= 0 {

                break

            };

            // 同期情報の場合に処理
            if read_buffer[0] == 0x47 {

                // TsPacket取り込み
                tpk = TsPacket{
                    sync: read_buffer[0],
                    transport_error_indicator: ((read_buffer[1] & 0x80) >> 7) as i32,
                    payload_unit_start_indicator: ((read_buffer[1] & 0x40) >> 6) as i32,
                    transport_priority: ((read_buffer[1] & 0x20) >> 5) as i32,
                    pid: ((read_buffer[1] as u32 & 0x1f) << 8) + read_buffer[2] as u32,
                    transport_scrambling_control: ((read_buffer[3] & 0xa0) >> 6) as i32,
                    adaptation_field_control: ((read_buffer[3] & 0x30) >> 4) as i32,
                    continuity_counter: (read_buffer[3] & 0x0f) as i32,
                    adaptation_field: 0,
                    payload: [0xff; TSPAYLOADMAX],
                    payloadlen: 184,
                    rcount: RCOUNT,
                };
        
                // アダプテーションフィールド制御情報でペイロード情報買い替え
                match tpk.adaptation_field_control {
                    // ヘッダー、アダプテーションフィールド、ペイロード
                    3 => {

                        let len = read_buffer[4];
                        if len >= 183 {
                            break;
                        }
                        payptr = 4 + len;
                        tpk.payloadlen -= len as i32 + 1;

                    },
                    // ヘッダー、アダプテーションフィールド
                    2 => {

                        tpk.payloadlen = LENGTH_PACKET as i32 - payptr as i32;

                    },
                    // ヘッダー、ペイロード
                    1 | _ => {

                        payptr = 4;

                    },
                };

                // ペイロードユニット開始インジケーターによりペイロード開始位置とペイロード長を調整
                if tpk.payload_unit_start_indicator == 1 {

                    payptr += 1;
                    tpk.payloadlen -= 1;

                };

                // ペイロード長が０以下、１８４を超える場合はスキップ
                if tpk.payloadlen <= 0 || tpk.payloadlen > 184 { 

                    return None

                };

                // ペイロードデータを構造体へコピー
                tpk.payload[..tpk.payloadlen as usize]
                    .copy_from_slice(&read_buffer[payptr as usize..payptr as usize +  tpk.payloadlen as usize]);  

                // デバッグ情報作成
                let _seclen = ((tpk.payload[1] as i32 & 0x0f) << 8) + tpk.payload[2] as i32 + 3;
                let _sid = ((tpk.payload[3] as i32 & 0xff) << 8) + tpk.payload[4] as i32;
                let _cur_next = tpk.payload[5] as i32 & 0x01;
                let _sec_num = tpk.payload[6] as i32;
                let _last_sec_num = tpk.payload[7] as i32;


                // 指定されたpidか確認
                for pid_cnt in 1..count {

                    // 指定されたpidとマッチする場合の処理
                    if secs[pid_cnt].pid == tpk.pid {

                        // TSパケット情報をsecs構造体へコピー
                        secs[pid_cnt].cur = tpk;

                        // pid初回のみの処理
                        //if secs[pid_cnt].cont == 0 && tpk.payload_unit_start_indicator == 1 {
                        if tpk.payload_unit_start_indicator == 1 {

                            // レングス情報を初期化
                            secs[pid_cnt].seclen = 0;
                            secs[pid_cnt].setlen = 0;
                            secs[pid_cnt].curlen = 0;

                            /* セクション長を調べる */
                            secs[pid_cnt].seclen = ((secs[pid_cnt].cur.payload[1] as i32 & 0x0f) << 8) + secs[pid_cnt].cur.payload[2] as i32 + 3; // ヘッダ

                            // セクション長が MAXSECLEN より長いときはこのセクションをスキップ
                            if secs[pid_cnt].seclen > MAXSECLEN as i32 {

                                secs[pid_cnt].cont = 0;

                                break;

                            };

                            // セクション長がTSパケットのペイロード長より長い場合の処理
                            if secs[pid_cnt].seclen > secs[pid_cnt].cur.payloadlen {

                                // セクションキャッシュにペイロードデータをコピー
                                secs[pid_cnt].buf[..secs[pid_cnt].cur.payloadlen as usize]
                                    .copy_from_slice(&secs[pid_cnt].cur.payload[..secs[pid_cnt].cur.payloadlen as usize]);

                                // レングス設定
                                secs[pid_cnt].setlen = secs[pid_cnt].cur.payloadlen;

                                // 処理済みフラグ設定
                                secs[pid_cnt].cont = 1;

                                // 継続パケットを処理
                                break;

                            };

                            // バッファーにペイロードデータをコピー
                            secs[pid_cnt].buf[..secs[pid_cnt].seclen as usize]
                                .copy_from_slice(&secs[pid_cnt].cur.payload[..secs[pid_cnt].seclen as usize]);

                            // レングス設定
                            secs[pid_cnt].setlen = secs[pid_cnt].seclen;

                            // カレントレングスへセクションレングスを退避
                            secs[pid_cnt].curlen = secs[pid_cnt].seclen;

                            // 処理済みフラグ設定
                            secs[pid_cnt].cont = 1;

                            // インデックス設定
                            _index = pid_cnt;

                            // 次のパケット処理
                            readbuff_file.consume(LENGTH_PACKET);

                            // リターン情報
                            return Some(secs[pid_cnt])

                        };

                        // pidの処理
                        // pidのセクションレングスからtsパケッド内のレングスを引いたレングスを計算
                        let len = secs[pid_cnt].seclen - secs[pid_cnt].setlen;

                        // 上記レングスが０以上の場合の処理
                        if len > 0 {

                            // TSパケットのペイロード長より長い場合の処理
                            if len > secs[pid_cnt].cur.payloadlen {

                                // ペイロードデータをコピー
                                secs[pid_cnt].buf[secs[pid_cnt].setlen as usize..secs[pid_cnt].setlen as usize + secs[pid_cnt].cur.payloadlen as usize]
                                    .copy_from_slice(&secs[pid_cnt].cur.payload[..secs[pid_cnt].cur.payloadlen as usize]);

                                // // レングス設定
                                secs[pid_cnt].setlen += secs[pid_cnt].cur.payloadlen;

                                // 継続パケットを処理
                                break;

                            };

                            // バッファーへペイロードデータをコピー
                            secs[pid_cnt].buf[secs[pid_cnt].setlen as usize..secs[pid_cnt].setlen as usize + len as usize]
                                .copy_from_slice(&secs[pid_cnt].cur.payload[..len as usize]);

                            // レングス設定
                            secs[pid_cnt].setlen = secs[pid_cnt].seclen;

                            // カレントレングスへセクションレングスを退避
                            secs[pid_cnt].curlen += len;

                            // 処理済みフラグ設定
                            secs[pid_cnt].cont = 1;

                            // インデックス設定
                            _index = pid_cnt;

                            // 次のパケットを処理
                            readbuff_file.consume(LENGTH_PACKET);
                            return Some(secs[pid_cnt]);

                        };
                    };
                }
            }

            // リターン情報（レングス）
            read_buffer.len()

        };

        // リードバッファクリア
        if length > 0 {

            readbuff_file.consume(length);

        };

    };

    // リターン情報（データ無し）
    None

}
