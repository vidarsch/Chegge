function drawGolfer(x, y, rotation) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(rotation);
    
    // Draw golfer body
    ctx.fillStyle = 'red';
    ctx.fillRect(-20, -30, 40, 60); // body
    
    // Draw club
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(50, 0);
    ctx.strokeStyle = 'blue';
    ctx.lineWidth = 3;
    ctx.stroke();
    
    // Draw club head
    ctx.fillStyle = 'silver';
    ctx.fillRect(50, -5, 10, 10);
    
    ctx.restore();
}