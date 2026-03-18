import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-marketplace-escrow',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './marketplace-escrow.component.html',
  styleUrl: './marketplace-escrow.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceEscrowComponent {}
