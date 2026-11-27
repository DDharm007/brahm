import { fnB } from "./b";

export function fnA(): string {
    return "A" + fnB();
}
