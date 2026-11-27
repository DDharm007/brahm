import { fnA } from "./a";

export function fnB(): string {
    return "B" + fnA();
}
