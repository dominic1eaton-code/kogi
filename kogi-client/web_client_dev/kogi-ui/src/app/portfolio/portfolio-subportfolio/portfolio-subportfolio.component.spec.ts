import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioSubportfolioComponent } from './portfolio-subportfolio.component';

describe('PortfolioSubportfolioComponent', () => {
  let component: PortfolioSubportfolioComponent;
  let fixture: ComponentFixture<PortfolioSubportfolioComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioSubportfolioComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioSubportfolioComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
