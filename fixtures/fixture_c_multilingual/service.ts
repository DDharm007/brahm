export interface ServiceConfig {
    port: number;
    host: string;
}

export function startService(config: ServiceConfig): void {
    console.log(`Starting service on ${config.host}:${config.port}`);
}
