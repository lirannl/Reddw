/* @refresh reload */
import { render } from 'solid-js/web';

import './index.css';
import App from './App';

if (import.meta.env.DEV) import("@locator/runtime").then((m) => m.setup());

render(() => <App />, document.getElementById('root') as HTMLElement);
