import { SolitaConfigAnchor, SolitaConfigShank, SolitaHandlerResult } from './types';
import { Options as PrettierOptions } from 'prettier';
export declare function handleAnchor(config: SolitaConfigAnchor, prettierConfig?: PrettierOptions): Promise<SolitaHandlerResult>;
export declare function handleShank(config: SolitaConfigShank, prettierConfig?: PrettierOptions): Promise<SolitaHandlerResult>;
