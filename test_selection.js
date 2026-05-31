const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ headless: false });
  const page = await browser.newPage();

  // Navigate to the app
  await page.goto('http://localhost:8080');
  await page.waitForSelector('#canvas');

  console.log('Testing rectangle selection...');

  // Wait for canvas to be ready
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Get the canvas element
  const canvas = await page.$('#canvas');

  // Step 1: Create a rectangle
  console.log('Creating a rectangle...');
  await page.mouse.move(300, 200);
  await page.mouse.down();
  await page.mouse.move(400, 300);
  await page.mouse.up();
  await new Promise(resolve => setTimeout(resolve, 100));

  // Step 2: Select the Rectangle tool (toolbar button at position 2)
  console.log('Selecting Select tool...');
  await page.mouse.click(45, 25); // Click on Select tool (first button)
  await new Promise(resolve => setTimeout(resolve, 100));

  // Step 3: Click on the rectangle we just created
  console.log('Clicking on the rectangle to select it...');
  await page.mouse.click(350, 250); // Click inside the rectangle
  await new Promise(resolve => setTimeout(resolve, 200));

  // Check if anything was selected by examining the canvas
  const isSelected = await page.evaluate(() => {
    // Check if there's a selected shape by looking at the state
    return window.selected_shape !== undefined;
  });

  console.log('Rectangle selected:', isSelected);

  // Take a screenshot
  await page.screenshot({ path: 'canvas_test.png' });
  console.log('Screenshot saved as canvas_test.png');

  // Keep browser open for manual inspection
  await new Promise(resolve => setTimeout(resolve, 3000));

  await browser.close();
})();