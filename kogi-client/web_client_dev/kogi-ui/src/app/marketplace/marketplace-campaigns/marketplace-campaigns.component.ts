import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-marketplace-campaigns',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './marketplace-campaigns.component.html',
  styleUrl: './marketplace-campaigns.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceCampaignsComponent {}
