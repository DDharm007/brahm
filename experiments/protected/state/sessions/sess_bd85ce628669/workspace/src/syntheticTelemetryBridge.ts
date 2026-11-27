/**
 * BRAHM-SYNTHETIC-ARTIFACT: decoy_c34470c51dc68c6d
 * Template: tpl_src_ts_telemetry_bridge v1.0.0
 * Controlled Inspection Target: brahm-commerce-demo
 */

export interface SyntheticTraceEnvelope {
    readonly artifactId: string;
    readonly canaryReference: string;
    readonly syntheticLayer: true;
}

export function createSyntheticTraceEnvelope(): SyntheticTraceEnvelope {
    return {
        artifactId: "decoy_c34470c51dc68c6d",
        canaryReference: "BRAHM-CANARY-F7D7DBA6FFC3",
        syntheticLayer: true,
    };
}
