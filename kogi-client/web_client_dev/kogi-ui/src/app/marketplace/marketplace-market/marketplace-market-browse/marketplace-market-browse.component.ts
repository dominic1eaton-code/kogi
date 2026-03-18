import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-market-browse',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-market-browse.component.html',
  styleUrl: './marketplace-market-browse.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceMarketBrowseComponent {}
