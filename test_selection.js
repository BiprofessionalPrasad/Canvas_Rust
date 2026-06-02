const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();

  try {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('#canvas');

    console.log('Testing shape selection workflow...');

    // Wait for WASM to initialize
    await page.waitForFunction(() => window.__canvasApp !== undefined, { timeout: 10000 });
    await new Promise(resolve => setTimeout(resolve, 500));

    // Step 1: Draw a rectangle by dragging on the canvas
    console.log('Drawing a rectangle...');
    const canvas = await page.$('#canvas');
    const canvasBox = await canvas.boundingBox();
    const clickX = canvasBox.x + 300;
    const clickY = canvasBox.y + 200;

    await page.mouse.move(clickX, clickY);
    await page.mouse.down();
    await page.mouse.move(clickX + 100, clickY + 100, { steps: 10 });
    await page.mouse.up();
    await new Promise(resolve => setTimeout(resolve, 200));

    // Step 2: Verify rectangle was created by checking selected color
    const colorAfterDraw = await page.evaluate(() => {
      try {
        return window.__canvasApp.get_selected_color();
      } catch (e) {
        return 'ERROR: ' + e.message;
      }
    });
    console.log('Color after drawing:', colorAfterDraw);
    if (colorAfterDraw === '#E0E0E0') {
      console.log('PASS: Rectangle drawn and selected with default color');
    } else {
      console.log('FAIL: Expected #E0E0E0, got', colorAfterDraw);
    }

    // Step 3: Change the color of the selected rectangle
    await page.evaluate(() => {
      window.__canvasApp.set_selected_color('#FF0000');
    });
    await new Promise(resolve => setTimeout(resolve, 100));

    const newColor = await page.evaluate(() => {
      return window.__canvasApp.get_selected_color();
    });
    console.log('Color after change:', newColor);
    if (newColor === '#ff0000') {
      console.log('PASS: Color changed to red');
    } else {
      console.log('FAIL: Expected #ff0000, got', newColor);
    }

    // Step 4: Change text on the selected shape (should no-op for rectangle)
    await page.evaluate(() => {
      window.__canvasApp.set_selected_text('Hello');
    });
    const textAfterSet = await page.evaluate(() => {
      return window.__canvasApp.get_selected_text();
    });
    console.log('Text after setting on rectangle:', textAfterSet || '(empty)');
    if (textAfterSet === 'Hello') {
      console.log('PASS: Text set on shape');
    } else {
      console.log('INFO: Text on non-text shape returned:', textAfterSet || '(empty)');
    }

    // Step 5: Verify font size getter works
    const fontSize = await page.evaluate(() => {
      return window.__canvasApp.get_selected_font_size();
    });
    console.log('Font size of selected shape:', fontSize);
    if (typeof fontSize === 'number' && fontSize > 0) {
      console.log('PASS: Font size getter works');
    } else {
      console.log('FAIL: Unexpected font size:', fontSize);
    }

    // Take a screenshot
    await page.screenshot({ path: 'canvas_test.png' });
    console.log('Screenshot saved as canvas_test.png');

    console.log('\nAll tests completed.');
  } catch (error) {
    console.error('Test failed:', error.message);
    process.exit(1);
  } finally {
    await browser.close();
  }
})();
