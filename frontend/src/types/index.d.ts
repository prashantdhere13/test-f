import { timezone } from 'dayjs/plugin/timezone.js';
import type { JwtPayload } from 'jwt-decode'
import type { AdvancedConfig } from './advanced_config'
import type { PlayoutConfig, Playlist as Ply, Quality } from './playout_config'

export {}

declare global {
    interface JwtPayloadExt extends JwtPayload {
        id: number
        channels: number[]
        role: string
    }

    interface PlaylistExt extends Ply {
        startInSec: number,
        lengthInSec: number
    }

    interface SecondaryEventsStyle {
        position?: string
        size?: number
        fontsize?: number
        x?: string | number
        y?: string | number
        boxcolor?: string
        boxborderw?: number
    }

    interface SecondaryEventsCfg {
        enable: boolean
        fontfile?: string
        styles?: Record<string, SecondaryEventsStyle>
        graphic_enable?: boolean
        graphic?: string
        graphic_position?: string
        graphic_scale?: number
    }

    interface PlayoutConfigExt extends PlayoutConfig {
        playlist: PlaylistExt
        quality?: Quality
        secondary_events?: SecondaryEventsCfg
        hls_variants?: HlsVariant[]
    }

    interface HlsVariant {
        name: string
        width: number
        height: number
        bitrate: string
        audio: string
    }

    interface PlayoutOutput {
        id: number
        name: string
        parameters: string
        channel_id: number
    }

    interface Token {
        access: string
        refresh: string
    }

    interface DataAuth {
        uuid: string
    }

    interface Channel {
        id: number
        extra_extensions: string | string[]
        name: string
        preview_url: string
        public: string
        playlists: string
        storage: string
        timezone?: string
    }

    interface User {
        id: number
        username: string
        mail?: string
        password?: string
        confirm?: string
        admin?: boolean
        channel_ids?: number[]
        role_id?: number
    }

    interface Crumb {
        text: string
        path: string
    }

    interface Payload {
        method: string
        headers: any
        body?: any
    }

    interface Playlist {
        channel: string
        date: string
        program: PlaylistItem[]
    }

    interface SecondaryEvent {
        template: string
        /** alias of in */
        offset: number
        duration: number
        /** mark-in seconds; falls back to offset */
        in?: number
        /** mark-out seconds = in + duration */
        out?: number
        text: string
    }

    interface PlaylistItem {
        date?: string
        uid: string
        begin: number
        title?: string | null
        asset_id?: string | null
        source: string
        duration: number
        in: number
        out: number
        audio?: string
        category?: string
        custom_filter?: string
        scte_marker_type?: 'splice_insert' | 'time_signal'
        scte_preroll?: number
        secondary_events?: SecondaryEvent[]
        overtime?: boolean
        segment?: number
        segment_count?: number
    }

    interface FileObject {
        name: string
        duration: number
    }

    interface MediaProbeStream {
        codec_type?: string
        codec_name?: string
        codec_long_name?: string
        profile?: string
        width?: number
        height?: number
        sample_aspect_ratio?: string
        display_aspect_ratio?: string
        field_order?: string
        r_frame_rate?: string
        tags?: {
            language?: string
            Language?: string
            title?: string
        }
    }

    interface MediaProbe {
        size?: number
        bit_rate?: number
        start_time?: number
        codec?: string
        profile?: string
        width?: number
        height?: number
        sample_aspect_ratio?: string
        display_aspect_ratio?: string
        field_order?: string
        r_frame_rate?: string
        audio?: MediaProbeStream[]
        subtitles?: MediaProbeStream[]
    }

    interface MediaAsset {
        asset_id: string
        title: string
        category: string
        media_type: string
        segments: number
        duration: string
        duration_seconds: number
        audio: number
        cc: boolean
        subtitle?: string
        status: string
        frame_rate: string
        path: string
        play_path?: string
        process_path?: string
        graphic_source?: string
        segment_length?: number
        probe?: MediaProbe
        tc_in?: number
        tc_out?: number
        total_duration_seconds?: number
        cut_segments?: { start: number; end: number }[]
    }

    interface Folder {
        uid: string
        name: string
    }

    interface FileFolderObject {
        source: string
        parent: string
        parent_folders: Folder[]
        folders: Folder[]
        files: FileObject[]
    }

    interface FolderObject {
        source: string
        parent: string
        folders: Folder[]
    }

    interface SourceObject {
        type: string
        src: string
    }

    interface TemplateItem {
        start: string
        duration: string
        shuffle: boolean
        paths: string[]
    }

    interface Template {
        sources: TemplateItem[]
    }

    interface BodyObject {
        paths?: string[]
        template?: Template
    }

    interface SystemStatistics {
        cpu: { cores: number; usage: number }
        load: { one: number; five: number; fifteen: number }
        memory: { total: number; used: number; free: number }
        network?: { name: string; current_in: number; current_out: number; total_in: number; total_out: number }
        storage?: { path: string; total: number; used: number }
        swap: { total: number; used: number; free: number }
        system: { name?: string; kernel?: string; version?: string; ffp_version?: string }
    }

    interface PlayoutStatus {
        media: PlaylistItem
        index: number
        ingest: boolean
        mode: string
        elapsed: number
        shift: number
        running?: boolean
        title?: string
    }

    interface SplitTime {
        id: number
        val: number
    }

    declare namespace Intl {
        type Key = "calendar" | "collation" | "currency" | "numberingSystem" | "timeZone" | "unit";

        function supportedValuesOf(input: Key): string[];
      }
}
