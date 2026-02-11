struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct InstanceInput {
    @location(2) instance_pos: vec2<f32>,
    @location(3) instance_color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) local_pos: vec2<f32>,
};

// 블록 버텍스 셰이더 — 인스턴스 색상 사용
@vertex
fn vs_block(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.position + instance.instance_pos;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.color = instance.instance_color;
    out.local_pos = model.position;
    return out;
}

// 선 버텍스 셰이더 — 버텍스 색상 사용
@vertex
fn vs_line(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 0.0, 1.0);
    out.color = model.color;
    out.local_pos = vec2<f32>(0.0, 0.0);
    return out;
}

// 카드 프래그먼트 셰이더 — 둥근 모서리 + 테두리 + 그림자
@fragment
fn fs_card(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = vec2<f32>(0.25, 0.15);
    let r = 0.022;
    let bw = 0.008;
    let shadow_off = vec2<f32>(0.006, -0.006);
    let shadow_blur = 0.016;

    // 그림자 SDF
    let sp = in.local_pos - shadow_off;
    let qs = abs(sp) - (half_size - vec2<f32>(r));
    let ds = length(max(qs, vec2<f32>(0.0))) - r;
    let shadow_a = (1.0 - smoothstep(0.0, shadow_blur, ds)) * 0.13;

    // 카드 외곽 SDF
    let qo = abs(in.local_pos) - (half_size - vec2<f32>(r));
    let d_o = length(max(qo, vec2<f32>(0.0))) - r;
    let aa = fwidth(d_o);
    let card_a = 1.0 - smoothstep(-aa * 0.5, aa * 0.5, d_o);

    // 카드 내부 SDF (테두리 판별용)
    let ih = half_size - vec2<f32>(bw);
    let ir = max(r - bw, 0.001);
    let qi = abs(in.local_pos) - (ih - vec2<f32>(ir));
    let d_i = length(max(qi, vec2<f32>(0.0))) - ir;

    // 최종 알파: 카드 + 그림자
    let total_a = card_a + shadow_a * (1.0 - card_a);
    if total_a < 0.001 {
        discard;
    }

    // 테두리 vs 채우기
    let is_fill = step(d_i, 0.0);
    let card_rgb = mix(in.color, vec3<f32>(1.0, 1.0, 1.0), is_fill);

    // 그림자(검정) 위에 카드 합성
    let final_rgb = card_rgb * (card_a / total_a);

    return vec4<f32>(final_rgb, total_a);
}

// 선 프래그먼트 셰이더
@fragment
fn fs_line(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

// UI 버텍스 셰이더 — 카메라 변환 없이 NDC 직접 출력
@vertex
fn vs_ui(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.color = model.color;
    out.local_pos = vec2<f32>(0.0, 0.0);
    return out;
}

// UI 프래그먼트 셰이더 — 단색 반투명 사각형
@fragment
fn fs_ui(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 0.92);
}
